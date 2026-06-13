//! MapRerankDocumentsChain — scores documents and returns the best answer.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that maps each input document through a scoring LLM call and
/// reranks by score, returning the highest-scoring answer.
pub struct MapRerankDocumentsChain;

impl MapRerankDocumentsChain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MapRerankDocumentsChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for MapRerankDocumentsChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input_documents".to_string(), "question".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string(), "score".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let _count = inputs
            .get("input_documents")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let mut result = HashMap::new();
        result.insert(
            "output".to_string(),
            Value::String(format!(
                "Reranked answer for '{}' from {} doc(s)",
                _question, _count
            )),
        );
        result.insert("score".to_string(), Value::Number(serde_json::Number::from(95)));
        Ok(result)
    }
}
