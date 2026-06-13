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

pub struct ReduceDocumentsChain {
    llm: Arc<dyn ChatModel>,
    map_prompt: PromptTemplate,
    reduce_prompt: PromptTemplate,
    document_variable_name: String,
    verbose: bool,
}

impl ReduceDocumentsChain {
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
impl Chain for ReduceDocumentsChain {
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
            info!("ReduceDocumentsChain mapping {} documents", docs.len());
        }

        let mut summaries = Vec::new();
        for doc in &docs {
            let mut kwargs = HashMap::new();
            kwargs.insert(
                self.document_variable_name.clone(),
                doc.page_content.clone(),
            );
            for (k, v) in &inputs {
                if k != "input_documents" && k != &self.document_variable_name {
                    let str_val = v
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| v.to_string());
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
            info!(
                "ReduceDocumentsChain reducing {} summaries",
                summaries.len()
            );
        }

        if summaries.is_empty() {
            let mut result = HashMap::new();
            result.insert("output_text".to_string(), Value::String(String::new()));
            return Ok(result);
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
        result.insert(
            "output_text".to_string(),
            Value::String(final_response.content),
        );
        Ok(result)
    }
}
