//! RunnablePick — picks specific keys from the input dict.

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RunnablePick {
    pub keys: Vec<String>,
}

impl RunnablePick {
    pub fn new(keys: Vec<String>) -> Self {
        Self { keys }
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnablePick {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();
        for key in &self.keys {
            if let Some(value) = input.get(key) {
                result.insert(key.clone(), value.clone());
            }
        }
        Ok(result)
    }
}
