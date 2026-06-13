//! Summarize chain implementation — uses LLM to summarize text.

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

/// A chain that prompts an LLM to produce a summary of the provided text.
pub struct SummarizeChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
}

impl fmt::Debug for SummarizeChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SummarizeChain")
            .field("prompt", &self.prompt)
            .finish_non_exhaustive()
    }
}

impl SummarizeChain {
    /// Creates a new `SummarizeChain` with the given LLM and prompt template.
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self { llm, prompt }
    }

    /// Runs the chain on the provided text, returning a summary.
    ///
    /// # Errors
    /// Returns [`ChainError`] if prompt formatting or the LLM call fails.
    pub async fn run(&self, text: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("text".to_string(), text.to_string());
        let prompt = self.prompt.format(&kwargs)?;
        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        Ok(response.content.trim().to_string())
    }
}

#[async_trait]
impl Chain for SummarizeChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["text".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let text = inputs
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let output = self.run(&text).await?;
        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(output));
        Ok(result)
    }
}
