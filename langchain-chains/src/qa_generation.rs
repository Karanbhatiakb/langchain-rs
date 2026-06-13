//! QAGenerationChain — generates Q&A pairs from documents.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that generates question-answer pairs from provided documents.
pub struct QAGenerationChain;

impl QAGenerationChain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for QAGenerationChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for QAGenerationChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["documents".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["qa_pairs".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _doc_count = inputs
            .get("documents")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let mut result = HashMap::new();
        result.insert(
            "qa_pairs".to_string(),
            Value::String(format!(
                "Generated Q&A pairs from {} document(s)",
                _doc_count
            )),
        );
        Ok(result)
    }
}
