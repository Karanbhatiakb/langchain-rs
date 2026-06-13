//! Rate limiting for LLM calls.
//!
//! Provides the [`RateLimiter`] trait and two implementations:
//! [`InMemoryRateLimiter`] (request-count based) and
//! [`TokenRateLimiter`] (token-count based).

use crate::errors::*;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

/// Trait for rate-limiting LLM calls.
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Acquires permission to proceed with a call. Returns an error if the
    /// rate limit has been exceeded.
    async fn acquire(&self) -> Result<()>;
    /// Resets the rate-limiter's internal counters and window.
    async fn reset(&self) -> Result<()>;
}

/// A simple in-memory rate limiter that tracks request counts per minute.
#[derive(Debug)]
pub struct InMemoryRateLimiter {
    /// Maximum number of requests allowed per minute.
    pub requests_per_minute: usize,
    current_count: Arc<AtomicUsize>,
    window_start: Arc<RwLock<Instant>>,
}

impl InMemoryRateLimiter {
    /// Creates a new `InMemoryRateLimiter` with the given requests-per-minute
    /// limit.
    pub fn new(requests_per_minute: usize) -> Self {
        Self {
            requests_per_minute,
            current_count: Arc::new(AtomicUsize::new(0)),
            window_start: Arc::new(RwLock::new(Instant::now())),
        }
    }
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn acquire(&self) -> Result<()> {
        let elapsed = {
            let start = self.window_start.read().unwrap();
            start.elapsed()
        };
        if elapsed >= std::time::Duration::from_secs(60) {
            {
                let mut start = self.window_start.write().unwrap();
                *start = Instant::now();
            }
            self.current_count.store(0, Ordering::SeqCst);
        }
        let count = self.current_count.fetch_add(1, Ordering::SeqCst);
        if count >= self.requests_per_minute {
            Err(ChainError::RateLimitError(format!(
                "Exceeded {} requests per minute",
                self.requests_per_minute
            )))
        } else {
            Ok(())
        }
    }

    async fn reset(&self) -> Result<()> {
        self.current_count.store(0, Ordering::SeqCst);
        {
            let mut start = self.window_start.write().unwrap();
            *start = Instant::now();
        }
        Ok(())
    }
}

/// A rate limiter that tracks token counts per minute.
#[derive(Debug)]
pub struct TokenRateLimiter {
    /// Maximum number of tokens allowed per minute.
    pub tokens_per_minute: usize,
    current_tokens: Arc<AtomicUsize>,
    window_start: Arc<RwLock<Instant>>,
}

impl TokenRateLimiter {
    /// Creates a new `TokenRateLimiter` with the given tokens-per-minute limit.
    pub fn new(tokens_per_minute: usize) -> Self {
        Self {
            tokens_per_minute,
            current_tokens: Arc::new(AtomicUsize::new(0)),
            window_start: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Records that `count` tokens have been consumed in the current window.
    pub async fn add_tokens(&self, count: usize) -> Result<()> {
        let elapsed = {
            let start = self.window_start.read().unwrap();
            start.elapsed()
        };
        if elapsed >= std::time::Duration::from_secs(60) {
            {
                let mut start = self.window_start.write().unwrap();
                *start = Instant::now();
            }
            self.current_tokens.store(0, Ordering::SeqCst);
        }
        let prev = self.current_tokens.fetch_add(count, Ordering::SeqCst);
        if prev + count > self.tokens_per_minute {
            Err(ChainError::RateLimitError(format!(
                "Exceeded {} tokens per minute",
                self.tokens_per_minute
            )))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl RateLimiter for TokenRateLimiter {
    async fn acquire(&self) -> Result<()> {
        self.add_tokens(1).await
    }

    async fn reset(&self) -> Result<()> {
        self.current_tokens.store(0, Ordering::SeqCst);
        {
            let mut start = self.window_start.write().unwrap();
            *start = Instant::now();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_rate_limiter_allows_under_limit() {
        let limiter = InMemoryRateLimiter::new(5);
        for _ in 0..5 {
            assert!(limiter.acquire().await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_blocks_over_limit() {
        let limiter = InMemoryRateLimiter::new(2);
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_err());
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_reset() {
        let limiter = InMemoryRateLimiter::new(1);
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_err());
        limiter.reset().await.unwrap();
        assert!(limiter.acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_token_rate_limiter_allows_under_limit() {
        let limiter = TokenRateLimiter::new(100);
        assert!(limiter.add_tokens(50).await.is_ok());
        assert!(limiter.add_tokens(49).await.is_ok());
    }

    #[tokio::test]
    async fn test_token_rate_limiter_blocks_over_limit() {
        let limiter = TokenRateLimiter::new(10);
        assert!(limiter.add_tokens(5).await.is_ok());
        assert!(limiter.add_tokens(10).await.is_err());
    }

    #[tokio::test]
    async fn test_token_rate_limiter_reset() {
        let limiter = TokenRateLimiter::new(5);
        assert!(limiter.add_tokens(5).await.is_ok());
        assert!(limiter.add_tokens(1).await.is_err());
        limiter.reset().await.unwrap();
        assert!(limiter.add_tokens(5).await.is_ok());
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_zero_limit() {
        let limiter = InMemoryRateLimiter::new(0);
        assert!(limiter.acquire().await.is_err());
    }

    #[tokio::test]
    async fn test_token_rate_limiter_acquire() {
        let limiter = TokenRateLimiter::new(5);
        for _ in 0..5 {
            assert!(limiter.acquire().await.is_ok());
        }
        assert!(limiter.acquire().await.is_err());
    }

    #[tokio::test]
    async fn test_token_rate_limiter_zero_limit() {
        let limiter = TokenRateLimiter::new(0);
        assert!(limiter.add_tokens(1).await.is_err());
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_edge_count() {
        let limiter = InMemoryRateLimiter::new(3);
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_err());
    }

    #[test]
    fn test_rate_limiter_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<InMemoryRateLimiter>();
        assert_sync::<InMemoryRateLimiter>();
        assert_send::<TokenRateLimiter>();
        assert_sync::<TokenRateLimiter>();
    }

    #[test]
    fn test_in_memory_rate_limiter_debug() {
        let limiter = InMemoryRateLimiter::new(10);
        let debug_str = format!("{:?}", limiter);
        assert!(debug_str.contains("InMemoryRateLimiter"));
    }

    #[test]
    fn test_token_rate_limiter_debug() {
        let limiter = TokenRateLimiter::new(100);
        let debug_str = format!("{:?}", limiter);
        assert!(debug_str.contains("TokenRateLimiter"));
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_exact_limit() {
        let limiter = InMemoryRateLimiter::new(3);
        for _ in 0..3 {
            assert!(limiter.acquire().await.is_ok());
        }
        assert!(limiter.acquire().await.is_err());
    }

    #[tokio::test]
    async fn test_token_rate_limiter_exact_limit() {
        let limiter = TokenRateLimiter::new(3);
        assert!(limiter.add_tokens(3).await.is_ok());
        assert!(limiter.add_tokens(1).await.is_err());
    }

    #[tokio::test]
    async fn test_token_rate_limiter_acquire_reset() {
        let limiter = TokenRateLimiter::new(3);
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_err());
        limiter.reset().await.unwrap();
        assert!(limiter.acquire().await.is_ok());
    }
}
