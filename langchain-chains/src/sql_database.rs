//! SQL Database chain implementation — uses LLM to generate SQL queries (stub execution).

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

/// A chain that prompts an LLM to produce SQL queries for a given database,
/// returning a stub result (no actual database execution is performed).
pub struct SQLDatabaseChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    database_url: String,
}

impl fmt::Debug for SQLDatabaseChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SQLDatabaseChain")
            .field("prompt", &self.prompt)
            .field("database_url", &self.database_url)
            .finish_non_exhaustive()
    }
}

impl SQLDatabaseChain {
    /// Creates a new `SQLDatabaseChain` with the given LLM, prompt template,
    /// and database URL.
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate, database_url: String) -> Self {
        Self {
            llm,
            prompt,
            database_url,
        }
    }

    /// Runs the chain on the provided query, returning a stub SQL result.
    ///
    /// # Errors
    /// Returns [`ChainError`] if prompt formatting or the LLM call fails.
    pub async fn run(&self, query: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("query".to_string(), query.to_string());
        kwargs.insert("database_url".to_string(), self.database_url.clone());
        let prompt = self.prompt.format(&kwargs)?;
        let messages = vec![HumanMessage::new(&prompt).into()];
        let _response = self.llm.predict_messages(&messages, None, None).await?;

        Ok("SQL query generated (stub)".to_string())
    }
}

#[async_trait]
impl Chain for SQLDatabaseChain {
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
