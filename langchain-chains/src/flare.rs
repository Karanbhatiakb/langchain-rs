//! Flare chain implementation — uses LLM with a retriever for active retrieval
//! augmented generation (stub).

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

/// A chain implementing the FLARE (Future-looking Active Retrieval) pattern,
/// which iteratively prompts the LLM and retrieves documents as needed.
///
/// This is a stub implementation that delegates to the LLM without actual
/// retrieval.
pub struct FlareChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    retriever_name: String,
}

impl fmt::Debug for FlareChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlareChain")
            .field("prompt", &self.prompt)
            .field("retriever_name", &self.retriever_name)
            .finish_non_exhaustive()
    }
}

impl FlareChain {
    /// Creates a new `FlareChain` with the given LLM and prompt template.
    ///
    /// A default retriever name is assigned; override it with
    /// [`FlareChain::with_retriever_name`].
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self {
            llm,
            prompt,
            retriever_name: "default".to_string(),
        }
    }

    /// Sets the retriever name (builder pattern).
    pub fn with_retriever_name(mut self, name: impl Into<String>) -> Self {
        self.retriever_name = name.into();
        self
    }

    /// Runs the chain on the provided query, returning the LLM's response.
    ///
    /// # Errors
    /// Returns [`ChainError`] if prompt formatting or the LLM call fails.
    pub async fn run(&self, query: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("query".to_string(), query.to_string());
        let prompt = self.prompt.format(&kwargs)?;
        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        Ok(response.content.trim().to_string())
    }
}

#[async_trait]
impl Chain for FlareChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["query".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let query = inputs
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let output = self.run(&query).await?;
        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(output));
        Ok(result)
    }
}
