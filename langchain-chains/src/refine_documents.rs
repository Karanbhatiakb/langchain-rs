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

    pub fn with_document_variable_name(mut self, name: impl Into<String>) -> Self {
        self.document_variable_name = name.into();
        self
    }

    pub fn with_initial_response_name(mut self, name: impl Into<String>) -> Self {
        self.initial_response_name = name.into();
        self
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
        kwargs.insert(
            self.document_variable_name.clone(),
            docs[0].page_content.clone(),
        );
        for (k, v) in &inputs {
            if k != "input_documents" {
                let str_val = v
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string());
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
            refine_kwargs.insert(
                self.document_variable_name.clone(),
                doc.page_content.clone(),
            );
            refine_kwargs.insert(
                self.initial_response_name.clone(),
                current_response.clone(),
            );
            for (k, v) in &inputs {
                if k != "input_documents"
                    && k != &self.document_variable_name
                    && k != &self.initial_response_name
                {
                    let str_val = v
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| v.to_string());
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
        result.insert(
            "output_text".to_string(),
            Value::String(current_response),
        );
        Ok(result)
    }
}
