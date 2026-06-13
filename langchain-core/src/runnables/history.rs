//! RunnableHistory — injects chat history into the input dict.

use crate::errors::*;
use crate::messages::BaseMessage;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type HistoryFn = Arc<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<Vec<BaseMessage>>> + Send>> + Send + Sync,
>;

/// A runnable that fetches chat history via an async function and inserts
/// the serialized result into the input dict at a configurable key.
pub struct RunnableHistory {
    history_fn: HistoryFn,
    /// The key at which the serialized chat history is inserted.
    pub history_key: String,
}

impl RunnableHistory {
    /// Creates a new `RunnableHistory` with the given async history function.
    ///
    /// The default key is `"history"`.
    pub fn new<F, Fut>(history_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Vec<BaseMessage>>> + Send + 'static,
    {
        Self {
            history_fn: Arc::new(move || Box::pin(history_fn())),
            history_key: "history".into(),
        }
    }

    /// Sets the key at which the serialized history will be inserted.
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.history_key = key.into();
        self
    }
}

impl Clone for RunnableHistory {
    fn clone(&self) -> Self {
        Self {
            history_fn: self.history_fn.clone(),
            history_key: self.history_key.clone(),
        }
    }
}

impl std::fmt::Debug for RunnableHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunnableHistory")
            .field("history_key", &self.history_key)
            .finish()
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnableHistory {
    async fn invoke(&self, mut input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let history = (self.history_fn)().await?;
        let json = serde_json::to_string(&history)?;
        input.insert(self.history_key.clone(), Value::String(json));
        Ok(input)
    }
}
