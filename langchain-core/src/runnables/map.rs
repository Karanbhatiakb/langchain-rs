//! RunnableMap — maps functions over inputs (alias for RunnableParallel).

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type MapFn = Arc<
    dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
>;

pub struct RunnableMap {
    steps: HashMap<String, MapFn>,
}

impl RunnableMap {
    pub fn new() -> Self {
        Self {
            steps: HashMap::new(),
        }
    }

    pub fn with_step<F, Fut>(mut self, key: impl Into<String>, f: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value>> + Send + 'static,
    {
        let key = key.into();
        let arc_f: MapFn = Arc::new(move |input: HashMap<String, Value>| Box::pin(f(input)));
        self.steps.insert(key, arc_f);
        self
    }
}

impl Default for RunnableMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RunnableMap {
    fn clone(&self) -> Self {
        Self {
            steps: self.steps.clone(),
        }
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnableMap {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();
        for (key, func) in &self.steps {
            let value = func(input.clone()).await?;
            result.insert(key.clone(), value);
        }
        Ok(result)
    }
}
