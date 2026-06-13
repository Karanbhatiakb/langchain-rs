//! LLM Bash chain implementation — uses LLM to generate shell commands (stub execution).

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

/// A chain that prompts an LLM to produce a shell command, then returns a
/// stub execution result (no actual shell execution is performed).
pub struct LLMBashChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    verbose: bool,
}

impl fmt::Debug for LLMBashChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LLMBashChain")
            .field("prompt", &self.prompt)
            .field("verbose", &self.verbose)
            .finish_non_exhaustive()
    }
}

impl LLMBashChain {
    /// Creates a new `LLMBashChain` with the given LLM and prompt template.
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self {
            llm,
            prompt,
            verbose: false,
        }
    }

    /// Runs the chain on the provided input, returning a stub command result.
    ///
    /// # Errors
    /// Returns [`ChainError`] if prompt formatting or the LLM call fails.
    pub async fn run(&self, input: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input.to_string());
        let prompt = self.prompt.format(&kwargs)?;

        if self.verbose {
            info!("LLMBashChain prompt: {}", prompt);
        }

        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let command = response.content.trim().to_string();

        Ok(format!("Command: {} (stub execution)", command))
    }
}

#[async_trait]
impl Chain for LLMBashChain {
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
