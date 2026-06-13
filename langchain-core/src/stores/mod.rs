//! Generic key-value store abstraction and in-memory implementation.

use crate::errors::*;
use async_trait::async_trait;
use dashmap::DashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Trait for key-value stores with batch operations.
///
/// # Type parameters
/// * `K` — Key type (must be `Send + Sync + 'static`).
/// * `V` — Value type (must be `Send + Sync + 'static`).
#[async_trait]
pub trait BaseStore<K, V>: Send + Sync
where
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    /// Retrieves values for the given keys (returns `None` for missing keys).
    async fn mget(&self, keys: Vec<K>) -> Result<Vec<Option<V>>>;
    /// Inserts or updates the given key-value pairs.
    async fn mset(&self, pairs: Vec<(K, V)>) -> Result<()>;
    /// Deletes the given keys from the store.
    async fn mdelete(&self, keys: Vec<K>) -> Result<()>;
    /// Returns all keys currently in the store.
    async fn yield_keys(&self) -> Result<Vec<K>>;
}

/// A concurrent in-memory store backed by a [`DashMap`].
///
/// # Type parameters
/// * `K` — Key type (must be `Eq + Hash + Send + Sync + 'static`).
/// * `V` — Value type (must be `Clone + Send + Sync + 'static`).
#[derive(Debug, Clone)]
pub struct InMemoryStore<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    store: Arc<DashMap<K, V>>,
}

impl<K, V> InMemoryStore<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Creates a new empty `InMemoryStore`.
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }
}

impl<K, V> Default for InMemoryStore<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<K, V> BaseStore<K, V> for InMemoryStore<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn mget(&self, keys: Vec<K>) -> Result<Vec<Option<V>>> {
        Ok(keys
            .into_iter()
            .map(|k| self.store.get(&k).map(|v| v.value().clone()))
            .collect())
    }

    async fn mset(&self, pairs: Vec<(K, V)>) -> Result<()> {
        for (k, v) in pairs {
            self.store.insert(k, v);
        }
        Ok(())
    }

    async fn mdelete(&self, keys: Vec<K>) -> Result<()> {
        for k in keys {
            self.store.remove(&k);
        }
        Ok(())
    }

    async fn yield_keys(&self) -> Result<Vec<K>> {
        Ok(self.store.iter().map(|r| r.key().clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_store_mset_mget() {
        let store = InMemoryStore::new();
        store.mset(vec![("a".to_string(), 1), ("b".to_string(), 2)]).await.unwrap();
        let results = store.mget(vec!["a".to_string(), "b".to_string(), "c".to_string()]).await.unwrap();
        assert_eq!(results[0], Some(1));
        assert_eq!(results[1], Some(2));
        assert_eq!(results[2], None);
    }

    #[tokio::test]
    async fn test_in_memory_store_mdelete() {
        let store = InMemoryStore::new();
        store.mset(vec![("x".to_string(), "val".to_string())]).await.unwrap();
        store.mdelete(vec!["x".to_string()]).await.unwrap();
        let results = store.mget(vec!["x".to_string()]).await.unwrap();
        assert!(results[0].is_none());
    }

    #[tokio::test]
    async fn test_in_memory_store_yield_keys() {
        let store = InMemoryStore::new();
        store.mset(vec![("k1".to_string(), 1), ("k2".to_string(), 2)]).await.unwrap();
        let mut keys = store.yield_keys().await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["k1".to_string(), "k2".to_string()]);
    }

    #[tokio::test]
    async fn test_in_memory_store_empty() {
        let store: InMemoryStore<String, String> = InMemoryStore::new();
        let keys = store.yield_keys().await.unwrap();
        assert!(keys.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_store_overwrite() {
        let store = InMemoryStore::new();
        store.mset(vec![("k".to_string(), "v1".to_string())]).await.unwrap();
        store.mset(vec![("k".to_string(), "v2".to_string())]).await.unwrap();
        let results = store.mget(vec!["k".to_string()]).await.unwrap();
        assert_eq!(results[0], Some("v2".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_store_default_empty() {
        let store: InMemoryStore<i32, i32> = InMemoryStore::default();
        let keys = store.yield_keys().await.unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_in_memory_store_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<InMemoryStore<String, String>>();
        assert_sync::<InMemoryStore<String, String>>();
    }
}
