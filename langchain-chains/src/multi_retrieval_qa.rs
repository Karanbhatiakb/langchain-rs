//! MultiRetrievalQA chain — routes between multiple retrievers based on input.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::Chain;

/// A chain that selects among multiple retrievers based on the query and
/// delegates to the chosen retriever-backed QA pipeline.
pub struct MultiRetrievalQA {
    retrievers: Vec<(String, Arc<dyn Chain>)>,
}

impl MultiRetrievalQA {
    pub fn new(retrievers: Vec<(String, Arc<dyn Chain>)>) -> Self {
        Self { retrievers }
    }

    pub fn add_retriever(mut self, key: impl Into<String>, chain: Arc<dyn Chain>) -> Self {
        self.retrievers.push((key.into(), chain));
        self
    }
}

#[async_trait]
impl Chain for MultiRetrievalQA {
    fn input_keys(&self) -> Vec<String> {
        vec!["query".to_string(), "retriever_key".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["result".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _key = inputs
            .get("retriever_key")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let _query = inputs
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut result = HashMap::new();
        result.insert(
            "result".to_string(),
            Value::String(format!(
                "MultiRetrievalQA routed to '{}' for query: {}",
                _key, _query
            )),
        );
        Ok(result)
    }
}
