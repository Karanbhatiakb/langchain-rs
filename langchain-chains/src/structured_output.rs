//! Structured Output chain implementation — uses LLM to produce JSON-structured output.

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

/// A chain that prompts an LLM to produce output conforming to a JSON schema,
/// then parses the response as a [`serde_json::Value`].
pub struct StructuredOutputChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    output_schema: serde_json::Value,
}

impl fmt::Debug for StructuredOutputChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StructuredOutputChain")
            .field("prompt", &self.prompt)
            .field("output_schema", &self.output_schema)
            .finish_non_exhaustive()
    }
}

impl StructuredOutputChain {
    /// Creates a new `StructuredOutputChain` with the given LLM, prompt template,
    /// and output JSON schema.
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate, output_schema: serde_json::Value) -> Self {
        Self {
            llm,
            prompt,
            output_schema,
        }
    }

    /// Runs the chain on the provided input, returning parsed JSON output.
    ///
    /// # Errors
    /// Returns [`ChainError`] if prompt formatting, the LLM call, or JSON parsing fails.
    pub async fn run(&self, input: &str) -> Result<serde_json::Value> {
        let schema_str = serde_json::to_string(&self.output_schema)
            .map_err(|e| ChainError::SerializationError(e.to_string()))?;

        let mut kwargs = HashMap::new();
        kwargs.insert("input".to_string(), input.to_string());
        kwargs.insert("schema".to_string(), schema_str);
        let prompt = self.prompt.format(&kwargs)?;
        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let content = response.content.trim().to_string();
        serde_json::from_str(&content).map_err(|e| ChainError::ParserError(e.to_string()))
    }
}

#[async_trait]
impl Chain for StructuredOutputChain {
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
        result.insert("output".to_string(), output);
        Ok(result)
    }
}
