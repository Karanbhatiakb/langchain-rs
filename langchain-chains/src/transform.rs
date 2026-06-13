//! Transform chain implementation.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::Chain;

pub struct TransformChain {
    input_variables: Vec<String>,
    output_variables: Vec<String>,
    transform_fn: Arc<dyn Fn(HashMap<String, Value>) -> Result<HashMap<String, Value>> + Send + Sync>,
}

impl TransformChain {
    pub fn new(
        input_variables: Vec<String>,
        output_variables: Vec<String>,
        transform_fn: Arc<dyn Fn(HashMap<String, Value>) -> Result<HashMap<String, Value>> + Send + Sync>,
    ) -> Self {
        Self {
            input_variables,
            output_variables,
            transform_fn,
        }
    }
}

#[async_trait]
impl Chain for TransformChain {
    fn input_keys(&self) -> Vec<String> {
        self.input_variables.clone()
    }

    fn output_keys(&self) -> Vec<String> {
        self.output_variables.clone()
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let filtered: HashMap<String, Value> = inputs
            .into_iter()
            .filter(|(k, _)| self.input_variables.contains(k))
            .collect();

        (self.transform_fn)(filtered)
    }
}
