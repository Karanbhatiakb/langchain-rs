//! Map-rerank chain implementation — maps documents with scoring, returns highest-scored answer.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::{HumanMessage, SystemMessage};
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct MapRerankChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    document_variable_name: String,
    verbose: bool,
}

impl MapRerankChain {
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

    pub fn default_prompt() -> PromptTemplate {
        PromptTemplate::from_template(
            "Use the following portion of context to answer the question.\n\n\
            Context: {context}\n\n\
            Question: {question}\n\n\
            Provide your answer and a relevance score from 0 to 100.\n\n\
            Respond in the following format:\n\
            Answer: <your answer>\n\
            Relevance Score: <number between 0 and 100>",
        )
    }

    fn parse_rerank_output(&self, text: &str) -> Result<(String, f64)> {
        let answer_re = Regex::new(r"(?s)Answer:\s*(.*?)(?:\n\s*Relevance Score:|$)")
            .map_err(|e| ChainError::ParserError(e.to_string()))?;
        let score_re = Regex::new(r"Relevance Score:\s*(\d+(?:\.\d+)?)")
            .map_err(|e| ChainError::ParserError(e.to_string()))?;

        let answer = answer_re
            .captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        let score = score_re
            .captures(text)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);

        Ok((answer, score))
    }
}

#[async_trait]
impl Chain for MapRerankChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["input_documents".to_string(), "question".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string(), "score".to_string()]
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
            info!("MapRerankChain processing {} documents", documents.len());
        }

        let mut best_answer = String::new();
        let mut best_score = f64::MIN;

        for doc in &documents {
            let mut kwargs = HashMap::new();
            kwargs.insert(self.document_variable_name.clone(), doc.clone());
            kwargs.insert("question".to_string(), question.clone());

            let formatted = self.prompt.format(&kwargs)?;

            let messages = vec![
                SystemMessage::new("Answer the question based on the context. Include a relevance score.").into(),
                HumanMessage::new(&formatted).into(),
            ];
            let response = self.llm.predict_messages(&messages, None, None).await?;

            let (answer, score) = self.parse_rerank_output(&response.content)?;

            if self.verbose {
                info!("MapRerankChain document score: {}", score);
            }

            if score > best_score {
                best_score = score;
                best_answer = answer;
            }
        }

        if best_score == f64::MIN {
            best_score = 0.0;
        }

        let mut result = HashMap::new();
        result.insert("output".to_string(), Value::String(best_answer));
        result.insert(
            "score".to_string(),
            Value::Number(
                serde_json::Number::from_f64(best_score)
                    .unwrap_or(serde_json::Number::from(0)),
            ),
        );
        Ok(result)
    }
}
