//! Combine documents chain implementation.

use async_trait::async_trait;
use langchain_core::documents::Document;
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
    verbose: bool,
}

impl StuffDocumentsChain {
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self {
            llm,
            prompt,
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

    fn combine_docs(&self, docs: &[Document]) -> String {
        docs.iter()
            .enumerate()
            .map(|(i, d)| format!("[{}] {}", i + 1, d.page_content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[async_trait]
impl Chain for StuffDocumentsChain {
    fn input_keys(&self) -> Vec<String> {
        vec![
            "input_documents".to_string(),
            self.document_variable_name.clone(),
        ]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output_text".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let docs: Vec<Document> = inputs
            .get("input_documents")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let combined = self.combine_docs(&docs);

        if self.verbose {
            info!("StuffDocumentsChain combining {} documents", docs.len());
        }

        let mut kwargs = HashMap::new();
        kwargs.insert(self.document_variable_name.clone(), combined);

        for (k, v) in &inputs {
            if k != "input_documents" && k != &self.document_variable_name {
                let str_val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                kwargs.insert(k.clone(), str_val);
            }
        }

        let formatted_prompt = self.prompt.format(&kwargs)?;

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&formatted_prompt).into()], None, None)
            .await?;

        let mut result = HashMap::new();
        result.insert("output_text".to_string(), Value::String(response.content));
        Ok(result)
    }
}

pub struct MapReduceDocumentsChain {
    llm: Arc<dyn ChatModel>,
    map_prompt: PromptTemplate,
    reduce_prompt: PromptTemplate,
    document_variable_name: String,
    verbose: bool,
}

impl MapReduceDocumentsChain {
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
}

#[async_trait]
impl Chain for MapReduceDocumentsChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input_documents".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output_text".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let docs: Vec<Document> = inputs
            .get("input_documents")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        if self.verbose {
            info!("MapReduceDocumentsChain mapping {} documents", docs.len());
        }

        let mut summaries = Vec::new();
        for doc in &docs {
            let mut kwargs = HashMap::new();
            kwargs.insert(self.document_variable_name.clone(), doc.page_content.clone());
            // Add additional inputs from the original input
            for (k, v) in &inputs {
                if k != "input_documents" && k != &self.document_variable_name {
                    let str_val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                    kwargs.insert(k.clone(), str_val);
                }
            }
            let map_prompt = self.map_prompt.format(&kwargs)?;

            let response = self
                .llm
                .predict_messages(&[HumanMessage::new(&map_prompt).into()], None, None)
                .await?;

            summaries.push(response.content);
        }

        if self.verbose {
            info!("MapReduceDocumentsChain reducing {} summaries", summaries.len());
        }

        let combined = summaries.join("\n\n");

        let mut reduce_kwargs = HashMap::new();
        reduce_kwargs.insert(self.document_variable_name.clone(), combined);
        let reduce_prompt = self.reduce_prompt.format(&reduce_kwargs)?;

        let final_response = self
            .llm
            .predict_messages(&[HumanMessage::new(&reduce_prompt).into()], None, None)
            .await?;

        let mut result = HashMap::new();
        result.insert("output_text".to_string(), Value::String(final_response.content));
        Ok(result)
    }
}

pub struct RefineDocumentsChain {
    llm: Arc<dyn ChatModel>,
    initial_prompt: PromptTemplate,
    refine_prompt: PromptTemplate,
    document_variable_name: String,
    initial_response_name: String,
    verbose: bool,
}

impl RefineDocumentsChain {
    pub fn new(
        llm: Arc<dyn ChatModel>,
        initial_prompt: PromptTemplate,
        refine_prompt: PromptTemplate,
    ) -> Self {
        Self {
            llm,
            initial_prompt,
            refine_prompt,
            document_variable_name: "context".to_string(),
            initial_response_name: "existing_answer".to_string(),
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for RefineDocumentsChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input_documents".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output_text".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let docs: Vec<Document> = inputs
            .get("input_documents")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        if self.verbose {
            info!("RefineDocumentsChain processing {} documents", docs.len());
        }

        if docs.is_empty() {
            let mut result = HashMap::new();
            result.insert("output_text".to_string(), Value::String(String::new()));
            return Ok(result);
        }

        let mut kwargs = HashMap::new();
        kwargs.insert(self.document_variable_name.clone(), docs[0].page_content.clone());
        for (k, v) in &inputs {
            if k != "input_documents" {
                let str_val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                kwargs.insert(k.clone(), str_val);
            }
        }
        let initial_prompt = self.initial_prompt.format(&kwargs)?;

        let mut current_response = self
            .llm
            .predict_messages(&[HumanMessage::new(&initial_prompt).into()], None, None)
            .await?
            .content;

        for doc in &docs[1..] {
            let mut refine_kwargs = HashMap::new();
            refine_kwargs.insert(self.document_variable_name.clone(), doc.page_content.clone());
            refine_kwargs.insert(self.initial_response_name.clone(), current_response.clone());
            for (k, v) in &inputs {
                if k != "input_documents" && k != &self.document_variable_name && k != &self.initial_response_name {
                    let str_val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                    refine_kwargs.insert(k.clone(), str_val);
                }
            }
            let refine_prompt = self.refine_prompt.format(&refine_kwargs)?;

            current_response = self
                .llm
                .predict_messages(&[HumanMessage::new(&refine_prompt).into()], None, None)
                .await?
                .content;
        }

        let mut result = HashMap::new();
        result.insert("output_text".to_string(), Value::String(current_response));
        Ok(result)
    }
}

pub struct MapRerankDocumentsChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    document_variable_name: String,
    score_key: String,
    verbose: bool,
}

impl MapRerankDocumentsChain {
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self {
            llm,
            prompt,
            document_variable_name: "context".to_string(),
            score_key: "score".to_string(),
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for MapRerankDocumentsChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input_documents".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output_text".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let docs: Vec<Document> = inputs
            .get("input_documents")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        if self.verbose {
            info!("MapRerankDocumentsChain reranking {} documents", docs.len());
        }

        let mut best_score = f64::MIN;
        let mut best_output = String::new();

        for doc in &docs {
            let mut kwargs = HashMap::new();
            kwargs.insert(self.document_variable_name.clone(), doc.page_content.clone());
            for (k, v) in &inputs {
                if k != "input_documents" && k != &self.document_variable_name {
                    let str_val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                    kwargs.insert(k.clone(), str_val);
                }
            }
            let formatted = self.prompt.format(&kwargs)?;

            let response = self
                .llm
                .predict_messages(&[HumanMessage::new(&formatted).into()], None, None)
                .await?;

            let score = response
                .additional_kwargs
                .get(&self.score_key)
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            if score > best_score {
                best_score = score;
                best_output = response.content;
            }
        }

        let mut result = HashMap::new();
        result.insert("output_text".to_string(), Value::String(best_output));
        result.insert(
            "score".to_string(),
            Value::Number(serde_json::Number::from_f64(best_score).unwrap_or(serde_json::Number::from(0))),
        );
        Ok(result)
    }
}
