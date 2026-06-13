//! LLM Symbolic Math chain implementation — uses LLM to solve symbolic math problems.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::types::Chain;

/// A chain that prompts an LLM to solve symbolic math expressions and returns
/// the model's response.
pub struct LLMSymbolicMathChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
}

impl fmt::Debug for LLMSymbolicMathChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LLMSymbolicMathChain")
            .field("prompt", &self.prompt)
            .finish_non_exhaustive()
    }
}

impl LLMSymbolicMathChain {
    /// Creates a new `LLMSymbolicMathChain` with the given LLM and prompt template.
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self { llm, prompt }
    }

    /// Runs the chain on the provided input, returning the LLM's symbolic math result.
    ///
    /// # Errors
    /// Returns [`ChainError`] if prompt formatting or the LLM call fails.
    pub async fn run(&self, input: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input.to_string());
        let prompt = self.prompt.format(&kwargs)?;
        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        Ok(response.content.trim().to_string())
    }
}

#[async_trait]
impl Chain for LLMSymbolicMathChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let output = self.run(&input).await?;
        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(output));
        Ok(result)
    }
}
