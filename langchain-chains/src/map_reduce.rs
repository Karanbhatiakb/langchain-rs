//! Map-reduce chain implementation — maps chunks through LLM then reduces results.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct MapReduceChain {
    llm: Arc<dyn ChatModel>,
    map_prompt: PromptTemplate,
    reduce_prompt: PromptTemplate,
    document_variable_name: String,
    verbose: bool,
}

impl MapReduceChain {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        map_prompt: PromptTemplate,
        reduce_prompt: PromptTemplate,
    ) -> Self {
        Self {
            llm,
            map_prompt,
            reduce_prompt,
            document_variable_name: "context".to_string(),
            verbose: false,
        }
    }

    pub fn with_document_variable_name(mut self, name: impl Into<String>) -> Self {
        self.document_variable_name = name.into();
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn default_map_prompt() -> PromptTemplate {
        PromptTemplate::from_template(
            "Given the following text, extract the key information relevant to the question.\n\n\
            Text: {context}\n\n\
            Question: {question}\n\n\
            Key information:",
        )
    }

    pub fn default_reduce_prompt() -> PromptTemplate {
        PromptTemplate::from_template(
            "Given the following extracted information, provide a comprehensive answer to the question.\n\n\
            Extracted information:\n{context}\n\n\
            Question: {question}\n\n\
            Answer:",
        )
    }
}

#[async_trait]
impl Chain for MapReduceChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input_documents".to_string(), "question".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let documents: Vec<String> = inputs
            .get("input_documents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if self.verbose {
            info!("MapReduceChain mapping {} documents", documents.len());
        }

        let mut map_results = Vec::new();
        for doc in &documents {
            let mut kwargs = HashMap::new();
            kwargs.insert(self.document_variable_name.clone(), doc.clone());
            kwargs.insert("question".to_string(), question.clone());
            let formatted = self.map_prompt.format(&kwargs)?;

            let messages = vec![HumanMessage::new(&formatted).into()];
            let response = self.llm.predict_messages(&messages, None, None).await?;
            map_results.push(response.content);
        }

        if map_results.is_empty() {
            let mut result = HashMap::new();
            result.insert("output".to_string(), Value::String(String::new()));
            return Ok(result);
        }

        if self.verbose {
            info!("MapReduceChain reducing {} results", map_results.len());
        }

        let combined = map_results.join("\n\n");

        let mut reduce_kwargs = HashMap::new();
        reduce_kwargs.insert(self.document_variable_name.clone(), combined);
        reduce_kwargs.insert("question".to_string(), question);
        let reduce_formatted = self.reduce_prompt.format(&reduce_kwargs)?;

        let messages = vec![HumanMessage::new(&reduce_formatted).into()];
        let final_response = self.llm.predict_messages(&messages, None, None).await?;

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(final_response.content));
        Ok(result)
    }
}
