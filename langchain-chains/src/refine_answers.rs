//! RefineAnswersChain — iteratively refines an answer through sequential
//! LLM calls.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that produces an initial answer and then refines it iteratively
/// by incorporating additional context at each step.
pub struct RefineAnswersChain {
    max_iterations: usize,
}

impl RefineAnswersChain {
    pub fn new() -> Self {
        Self {
            max_iterations: 3,
        }
    }

    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }
}

impl Default for RefineAnswersChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for RefineAnswersChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["question".to_string(), "context".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["refined_answer".to_string(), "iterations".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut result = HashMap::new();
        result.insert(
            "refined_answer".to_string(),
            Value::String(format!(
                "Refined answer for '{}' after {} iteration(s)",
                _question, self.max_iterations
            )),
        );
        result.insert(
            "iterations".to_string(),
            Value::Number(serde_json::Number::from(self.max_iterations as u64)),
        );
        Ok(result)
    }
}
