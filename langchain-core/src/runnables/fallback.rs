//! RunnableFallbacks — tries a primary runnable, then fallbacks on error.

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type RunFn = Arc<
    dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>>> + Send>>
        + Send
        + Sync,
>;

/// A runnable that tries a primary runnable first, and if it fails,
/// attempts each fallback runnable in order until one succeeds.
pub struct RunnableFallbacks {
    primary: RunFn,
    fallbacks: Vec<RunFn>,
}

impl RunnableFallbacks {
    /// Creates a new `RunnableFallbacks` with the given primary runnable.
    pub fn new<F, Fut>(primary: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        Self {
            primary: Arc::new(move |input: HashMap<String, Value>| Box::pin(primary(input))),
            fallbacks: Vec::new(),
        }
    }

    /// Adds a fallback runnable to the end of the fallback list.
    pub fn add_fallback<F, Fut>(mut self, fallback: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        self.fallbacks
            .push(Arc::new(move |input: HashMap<String, Value>| Box::pin(fallback(input))));
        self
    }
}

impl Clone for RunnableFallbacks {
    fn clone(&self) -> Self {
        Self {
            primary: self.primary.clone(),
            fallbacks: self.fallbacks.clone(),
        }
    }
}

impl std::fmt::Debug for RunnableFallbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunnableFallbacks")
            .field("fallbacks", &self.fallbacks.len())
            .finish()
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnableFallbacks {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        match (self.primary)(input.clone()).await {
            Ok(output) => Ok(output),
            Err(primary_err) => {
                for fallback in &self.fallbacks {
                    if let Ok(output) = (fallback)(input.clone()).await {
                        return Ok(output);
                    }
                }
                Err(primary_err)
            }
        }
    }
}
