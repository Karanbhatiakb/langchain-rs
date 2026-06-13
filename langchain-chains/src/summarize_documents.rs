//! SummarizeDocumentsChain — summarizes a collection of documents.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that produces a concise summary from a set of input documents.
pub struct SummarizeDocumentsChain;

impl SummarizeDocumentsChain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SummarizeDocumentsChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for SummarizeDocumentsChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input_documents".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["summary".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _count = inputs
            .get("input_documents")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let mut result = HashMap::new();
        result.insert(
            "summary".to_string(),
            Value::String(format!("Summary of {} document(s)", _count)),
        );
        Ok(result)
    }
}
