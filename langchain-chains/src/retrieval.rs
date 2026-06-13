//! Retrieval chain implementation.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use langchain_vectorstores::traits::VectorStore;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct RetrievalQA {
    llm: Arc<dyn ChatModel>,
    vectorstore: Arc<dyn VectorStore>,
    prompt: PromptTemplate,
    k: usize,
    verbose: bool,
}

impl RetrievalQA {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        vectorstore: Arc<dyn VectorStore>,
    ) -> Self {
        let prompt = PromptTemplate::from_template(
            "Use the following context to answer the question.\n\nContext: {context}\n\nQuestion: {question}\n\nAnswer:",
        );
        Self {
            llm,
            vectorstore,
            prompt,
            k: 4,
            verbose: false,
        }
    }

    pub fn with_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    async fn retrieve_docs(&self, query: &str) -> Result<Vec<Document>> {
        self.vectorstore
            .similarity_search(query, self.k)
            .await
    }
}

#[async_trait]
impl Chain for RetrievalQA {
    fn input_keys(&self) -> Vec<String> {
        vec!["query".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["result".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let query = inputs
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.verbose {
            info!("RetrievalQA query: {}", query);
        }

        let docs = self.retrieve_docs(&query).await?;

        let context: String = docs
            .iter()
            .enumerate()
            .map(|(i, d)| format!("[{}] {}", i + 1, d.page_content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let mut format_kwargs = HashMap::new();
        format_kwargs.insert("context".to_string(), context);
        format_kwargs.insert("question".to_string(), query);
        let formatted_prompt = self.prompt.format(&format_kwargs)?;

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&formatted_prompt).into()], None, None)
            .await?;

        let mut result = HashMap::new();
        result.insert("result".to_string(), Value::String(response.content));
        Ok(result)
    }
}

pub struct RetrievalQAWithSourcesChain {
    llm: Arc<dyn ChatModel>,
    vectorstore: Arc<dyn VectorStore>,
    prompt: PromptTemplate,
    k: usize,
    verbose: bool,
}

impl RetrievalQAWithSourcesChain {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        vectorstore: Arc<dyn VectorStore>,
    ) -> Self {
        let prompt = PromptTemplate::from_template(
            "Use the following context to answer the question. Include sources.\n\nContext: {context}\n\nQuestion: {question}\n\nAnswer (with sources):",
        );
        Self {
            llm,
            vectorstore,
            prompt,
            k: 4,
            verbose: false,
        }
    }

    pub fn with_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for RetrievalQAWithSourcesChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["query".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["answer".to_string(), "sources".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let query = inputs
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.verbose {
            info!("RetrievalQAWithSources query: {}", query);
        }

        let docs = self
            .vectorstore
            .similarity_search(&query, self.k)
            .await?;

        let context: String = docs
            .iter()
            .enumerate()
            .map(|(i, d)| format!("[{}] {} (source: {:?})", i + 1, d.page_content, d.metadata))
            .collect::<Vec<_>>()
            .join("\n\n");

        let sources: Vec<String> = docs
            .iter()
            .map(|d| {
                d.metadata
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            })
            .collect();

        let mut format_kwargs = HashMap::new();
        format_kwargs.insert("context".to_string(), context);
        format_kwargs.insert("question".to_string(), query);
        let formatted_prompt = self.prompt.format(&format_kwargs)?;

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&formatted_prompt).into()], None, None)
            .await?;

        let mut result = HashMap::new();
        result.insert("answer".to_string(), Value::String(response.content));
        result.insert("sources".to_string(), serde_json::to_value(sources).unwrap_or_default());
        Ok(result)
    }
}
