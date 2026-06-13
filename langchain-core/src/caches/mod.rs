//! Caching layer for LLM responses to reduce cost and latency.
//!
//! Provides the [`BaseCache`] trait and implementations for in-memory
//! ([`InMemoryCache`]) and Redis-backed ([`RedisCache`]) caching.

use crate::errors::*;
use async_trait::async_trait;
use dashmap::DashMap;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for LLM response caches.
///
/// Implementors store prompt+LLM pairs and return cached results.
#[async_trait]
pub trait BaseCache: Send + Sync {
    /// Looks up a cached response for the given prompt and LLM identifier.
    async fn lookup(&self, prompt: &str, llm: &str) -> Result<Option<String>>;
    /// Stores a response in the cache.
    async fn update(&self, prompt: &str, llm: &str, value: &str) -> Result<()>;
    /// Clears all cached entries.
    async fn clear(&self) -> Result<()>;
}

/// Creates a SHA-256 based cache key from the prompt and LLM identifier.
fn make_cache_key(prompt: &str, llm: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    hasher.update(llm.as_bytes());
    let hash = hasher.finalize();
    format!("langchain:cache:{}", hex::encode(hash))
}

/// An in-memory cache backed by a concurrent [`DashMap`].
#[derive(Debug, Clone)]
pub struct InMemoryCache {
    store: Arc<DashMap<String, String>>,
}

impl InMemoryCache {
    /// Creates a new empty `InMemoryCache`.
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseCache for InMemoryCache {
    async fn lookup(&self, prompt: &str, llm: &str) -> Result<Option<String>> {
        let key = make_cache_key(prompt, llm);
        Ok(self.store.get(&key).map(|v| v.value().clone()))
    }

    async fn update(&self, prompt: &str, llm: &str, value: &str) -> Result<()> {
        let key = make_cache_key(prompt, llm);
        self.store.insert(key, value.to_string());
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.store.clear();
        Ok(())
    }
}

/// A Redis-backed cache with optional TTL and custom key prefix.
///
/// Falls back to an in-memory cache when the `redis` feature is disabled.
#[derive(Debug)]
pub struct RedisCache {
    /// The Redis connection string.
    pub connection_string: String,
    /// Optional TTL in seconds for cached entries.
    pub ttl: Option<u64>,
    /// Key prefix for cache entries (default: `"langchain:cache"`).
    pub prefix: String,
    #[cfg(feature = "redis")]
    client: Option<redis::Client>,
    fallback_cache: Arc<RwLock<HashMap<String, Option<String>>>>,
}

impl Clone for RedisCache {
    fn clone(&self) -> Self {
        Self {
            connection_string: self.connection_string.clone(),
            ttl: self.ttl,
            prefix: self.prefix.clone(),
            #[cfg(feature = "redis")]
            client: self.client.clone(),
            fallback_cache: self.fallback_cache.clone(),
        }
    }
}

impl RedisCache {
    /// Creates a new `RedisCache` with the given connection string.
    pub fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            ttl: None,
            prefix: "langchain:cache".to_string(),
            #[cfg(feature = "redis")]
            client: Some(redis::Client::open(connection_string).unwrap_or_else(|_| {
                redis::Client::open("redis://127.0.0.1").unwrap()
            })),
            fallback_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sets the TTL (seconds) for cached entries (builder pattern).
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl = Some(ttl_seconds);
        self
    }

    /// Sets a custom key prefix (builder pattern).
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Logs a warning about using the fallback cache.
    fn warn_fallback() {
        tracing::warn!("Redis feature not enabled; using in-memory fallback cache");
    }

    /// Returns an async Redis connection.
    #[cfg(feature = "redis")]
    async fn get_connection(&self) -> Result<redis::aio::Connection> {
        let client = self.client.as_ref().ok_or_else(|| {
            ChainError::ConfigError("Redis client not initialized".into())
        })?;
        client
            .get_async_connection()
            .await
            .map_err(|e| ChainError::LLMError(format!("Redis connection error: {}", e)))
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl BaseCache for RedisCache {
    async fn lookup(&self, prompt: &str, llm: &str) -> Result<Option<String>> {
        let key = make_cache_key(prompt, llm);
        let mut conn = self.get_connection().await?;
        let result: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| ChainError::LLMError(format!("Redis GET error: {}", e)))?;
        Ok(result)
    }

    async fn update(&self, prompt: &str, llm: &str, value: &str) -> Result<()> {
        let key = make_cache_key(prompt, llm);
        let mut conn = self.get_connection().await?;
        if let Some(ttl) = self.ttl {
            redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl)
                .arg(value)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(|e| ChainError::LLMError(format!("Redis SETEX error: {}", e)))?;
        } else {
            redis::cmd("SET")
                .arg(&key)
                .arg(value)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(|e| ChainError::LLMError(format!("Redis SET error: {}", e)))?;
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let pattern = format!("{}:*", self.prefix);
        let mut cursor: u64 = 0;
        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| ChainError::LLMError(format!("Redis SCAN error: {}", e)))?;
            if !keys.is_empty() {
                redis::cmd("DEL")
                    .arg(&keys)
                    .query_async::<_, ()>(&mut conn)
                    .await
                    .map_err(|e| ChainError::LLMError(format!("Redis DEL error: {}", e)))?;
            }
            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(not(feature = "redis"))]
#[async_trait]
impl BaseCache for RedisCache {
    async fn lookup(&self, prompt: &str, llm: &str) -> Result<Option<String>> {
        Self::warn_fallback();
        let key = make_cache_key(prompt, llm);
        let cache = self.fallback_cache.read().map_err(|e| {
            ChainError::LLMError(format!("Fallback cache lock error: {}", e))
        })?;
        Ok(cache.get(&key).and_then(|v| v.clone()))
    }

    async fn update(&self, prompt: &str, llm: &str, value: &str) -> Result<()> {
        Self::warn_fallback();
        let key = make_cache_key(prompt, llm);
        let mut cache = self.fallback_cache.write().map_err(|e| {
            ChainError::LLMError(format!("Fallback cache lock error: {}", e))
        })?;
        cache.insert(key, Some(value.to_string()));
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        Self::warn_fallback();
        let mut cache = self.fallback_cache.write().map_err(|e| {
            ChainError::LLMError(format!("Fallback cache lock error: {}", e))
        })?;
        cache.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_cache_lookup_miss() {
        let cache = InMemoryCache::new();
        let result = cache.lookup("prompt", "llm").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_cache_update_and_lookup() {
        let cache = InMemoryCache::new();
        cache.update("hello", "gpt4", "world").await.unwrap();
        let result = cache.lookup("hello", "gpt4").await.unwrap();
        assert_eq!(result, Some("world".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_cache_different_llm() {
        let cache = InMemoryCache::new();
        cache.update("prompt", "gpt4", "response_a").await.unwrap();
        let result = cache.lookup("prompt", "claude").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_cache_clear() {
        let cache = InMemoryCache::new();
        cache.update("key", "llm", "val").await.unwrap();
        cache.clear().await.unwrap();
        let result = cache.lookup("key", "llm").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_cache_update_overwrites() {
        let cache = InMemoryCache::new();
        cache.update("k", "m", "v1").await.unwrap();
        cache.update("k", "m", "v2").await.unwrap();
        let result = cache.lookup("k", "m").await.unwrap();
        assert_eq!(result, Some("v2".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_cache_default() {
        let cache = InMemoryCache::default();
        cache.update("test", "model", "value").await.unwrap();
        let result = cache.lookup("test", "model").await.unwrap();
        assert_eq!(result, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_make_cache_key_deterministic() {
        let key1 = make_cache_key("hello", "world");
        let key2 = make_cache_key("hello", "world");
        assert_eq!(key1, key2);
        assert!(key1.starts_with("langchain:cache:"));
    }

    #[tokio::test]
    async fn test_make_cache_key_different_prompts() {
        let key1 = make_cache_key("hello", "model");
        let key2 = make_cache_key("world", "model");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_in_memory_cache_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<InMemoryCache>();
        assert_sync::<InMemoryCache>();
    }
}
