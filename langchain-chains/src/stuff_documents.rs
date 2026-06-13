//! Stuff documents chain implementation — combines all documents into a single prompt.

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

pub struct StuffDocumentsChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    document_variable_name: String,
    document_separator: String,
    verbose: bool,
}

impl StuffDocumentsChain {
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self {
            llm,
            prompt,
            document_variable_name: "context".to_string(),
            document_separator: "\n\n".to_string(),
            verbose: false,
        }
    }

    pub fn with_document_variable_name(mut self, name: impl Into<String>) -> Self {
        self.document_variable_name = name.into();
        self
    }

    pub fn with_document_separator(mut self, separator: impl Into<String>) -> Self {
        self.document_separator = separator.into();
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn default_qa_prompt() -> PromptTemplate {
        PromptTemplate::from_template(
            "Use the following pieces of context to answer the question at the end. \
            If you don't know the answer, just say that you don't know, don't try to make up an answer.\n\n\
            {context}\n\n\
            Question: {question}\n\n\
            Answer:",
        )
    }

    pub fn default_summarize_prompt() -> PromptTemplate {
        PromptTemplate::from_template(
            "Write a concise summary of the following:\n\n\
            {context}\n\n\
            Summary:",
        )
    }

    fn stuff_documents(&self, documents: &[String]) -> String {
        documents.join(&self.document_separator)
    }
}

#[async_trait]
impl Chain for StuffDocumentsChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input_documents".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
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
            info!("StuffDocumentsChain stuffing {} documents", documents.len());
        }

        let stuffed = self.stuff_documents(&documents);

        let mut kwargs = HashMap::new();
        kwargs.insert(self.document_variable_name.clone(), stuffed);

        for (k, v) in &inputs {
            if k != "input_documents" && k != &self.document_variable_name {
                let str_val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                kwargs.insert(k.clone(), str_val);
            }
        }

        let formatted = self.prompt.format(&kwargs)?;

        let messages = vec![HumanMessage::new(&formatted).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(response.content));
        Ok(result)
    }
}
