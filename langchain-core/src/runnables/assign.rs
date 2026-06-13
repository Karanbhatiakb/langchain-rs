//! RunnableAssign — assigns new keys to the input dict.

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct RunnableAssign {
    mapper: Arc<
        dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>>> + Send>>
            + Send
            + Sync,
    >,
}

impl RunnableAssign {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        Self {
            mapper: Arc::new(move |input: HashMap<String, Value>| Box::pin(f(input))),
        }
    }
}

impl Clone for RunnableAssign {
    fn clone(&self) -> Self {
        Self {
            mapper: self.mapper.clone(),
        }
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnableAssign {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut base = input;
        let new_keys = (self.mapper)(base.clone()).await?;
        for (k, v) in new_keys {
            base.insert(k, v);
        }
        Ok(base)
    }
}
