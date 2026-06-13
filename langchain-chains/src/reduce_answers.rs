//! ReduceAnswersChain — combines multiple parallel answers into a final answer.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that takes a list of candidate answers (produced in parallel) and
/// reduces/combines them into a single coherent final answer.
pub struct ReduceAnswersChain;

impl ReduceAnswersChain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReduceAnswersChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for ReduceAnswersChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["answers".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["final_answer".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _count = inputs
            .get("answers")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let mut result = HashMap::new();
        result.insert(
            "final_answer".to_string(),
            Value::String(format!("Reduced answer from {} candidate(s)", _count)),
        );
        Ok(result)
    }
}
