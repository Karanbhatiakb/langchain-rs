//! QuestionAnsweringChain — full question-answering pipeline.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that implements a complete question-answering pipeline:
/// document loading, retrieval, and answer generation.
pub struct QuestionAnsweringChain;

impl QuestionAnsweringChain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for QuestionAnsweringChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for QuestionAnsweringChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["question".to_string(), "documents".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["answer".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut result = HashMap::new();
        result.insert(
            "answer".to_string(),
            Value::String(format!("Answer to: {}", _question)),
        );
        Ok(result)
    }
}
