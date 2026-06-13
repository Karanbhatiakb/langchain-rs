//! RunnableRetry — retries a runnable with exponential backoff and jitter.

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::time::sleep;

type RunFn = Arc<
    dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>>> + Send>>
        + Send
        + Sync,
>;

/// A runnable that retries the wrapped runnable with exponential backoff
/// and jitter.
///
/// On each retry the delay is `base_delay_ms * 2^attempt` plus a random
/// jitter of `[0, base_delay_ms)`. After `max_retries` failed attempts the
/// last error is propagated.
pub struct RunnableRetry {
    runnable: RunFn,
    /// Maximum number of retry attempts.
    pub max_retries: usize,
    /// Base delay in milliseconds before the first retry.
    pub base_delay_ms: u64,
}

impl RunnableRetry {
    /// Creates a new `RunnableRetry`.
    ///
    /// * `runnable` — The async function to retry.
    /// * `max_retries` — Maximum number of retry attempts (0 = no retries).
    /// * `base_delay_ms` — Base delay in milliseconds for exponential backoff.
    pub fn new<F, Fut>(runnable: F, max_retries: usize, base_delay_ms: u64) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        Self {
            runnable: Arc::new(move |input: HashMap<String, Value>| Box::pin(runnable(input))),
            max_retries,
            base_delay_ms,
        }
    }
}

impl Clone for RunnableRetry {
    fn clone(&self) -> Self {
        Self {
            runnable: self.runnable.clone(),
            max_retries: self.max_retries,
            base_delay_ms: self.base_delay_ms,
        }
    }
}

impl std::fmt::Debug for RunnableRetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunnableRetry")
            .field("max_retries", &self.max_retries)
            .field("base_delay_ms", &self.base_delay_ms)
            .finish()
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnableRetry {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            match (self.runnable)(input.clone()).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    if attempt < self.max_retries {
                        let exp_delay = self.base_delay_ms * 2u64.pow(attempt as u32);
                        let jitter = rand::Rng::gen_range(
                            &mut rand::thread_rng(),
                            0..self.base_delay_ms.max(1),
                        );
                        sleep(std::time::Duration::from_millis(exp_delay + jitter)).await;
                    }
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| ChainError::LLMError("Retry failed with no error captured".into())))
    }
}
