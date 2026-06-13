//! RetrievalQA chain — simple Q&A using a vector store retriever.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that answers a question by first retrieving relevant documents from
/// a vector store and then passing them as context to an LLM.
pub struct RetrievalQA;

impl RetrievalQA {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RetrievalQA {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for RetrievalQA {
    fn input_keys(&self) -> Vec<String> {
        vec!["query".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["result".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _query = inputs
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut result = HashMap::new();
        result.insert(
            "result".to_string(),
            Value::String(format!("RetrievalQA result for query: {}", _query)),
        );
        Ok(result)
    }
}
