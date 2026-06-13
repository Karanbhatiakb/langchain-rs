//! QA with Sources chain implementation — uses LLM to answer questions and
//! cite sources.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::types::Chain;

/// The result of a [`QAWithSourcesChain`] call, containing the answer text and
/// a list of source identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAWithSourcesResult {
    /// The answer text produced by the LLM.
    pub answer: String,
    /// The source identifiers cited by the LLM.
    pub sources: Vec<String>,
}

/// A chain that prompts an LLM to answer a question and cite supporting sources.
pub struct QAWithSourcesChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
}

impl fmt::Debug for QAWithSourcesChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QAWithSourcesChain")
            .field("prompt", &self.prompt)
            .finish_non_exhaustive()
    }
}

impl QAWithSourcesChain {
    /// Creates a new `QAWithSourcesChain` with the given LLM and prompt template.
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self { llm, prompt }
    }

    /// Runs the chain on the provided question, returning an answer with sources.
    ///
    /// # Errors
    /// Returns [`ChainError`] if prompt formatting, the LLM call, or response
    /// parsing fails.
    pub async fn run(&self, question: &str) -> Result<QAWithSourcesResult> {
        let mut kwargs = HashMap::new();
        kwargs.insert("question".to_string(), question.to_string());
        let prompt = self.prompt.format(&kwargs)?;
        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let content = response.content.trim().to_string();

        let parsed: QAWithSourcesResult = serde_json::from_str(&content)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse QA result: {}", e)))?;

        Ok(parsed)
    }
}

#[async_trait]
impl Chain for QAWithSourcesChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["question".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["answer".to_string(), "sources".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let result = self.run(&question).await?;

        let mut output = HashMap::new();
        output.insert("answer".to_string(), Value::String(result.answer));
        output.insert(
            "sources".to_string(),
            Value::Array(
                result
                    .sources
                    .into_iter()
                    .map(Value::String)
                    .collect(),
            ),
        );
        Ok(output)
    }
}
