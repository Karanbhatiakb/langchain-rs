use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use langchain_vectorstores::traits::VectorStore;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct ChatVectorDBChain {
    llm: Arc<dyn ChatModel>,
    vectorstore: Arc<dyn VectorStore>,
    condense_question_prompt: PromptTemplate,
    combine_docs_prompt: PromptTemplate,
    k: usize,
    verbose: bool,
}

impl ChatVectorDBChain {
    pub fn new(llm: Arc<dyn ChatModel>, vectorstore: Arc<dyn VectorStore>) -> Self {
        let condense_question_prompt = PromptTemplate::from_template(
            "Given a chat history and a follow-up question, rephrase the follow-up question to be a standalone question.\n\nChat History:\n{chat_history}\n\nFollow-up Input: {question}\n\nStandalone question:",
        );
        let combine_docs_prompt = PromptTemplate::from_template(
            "Use the following pieces of context to answer the question at the end.\n\nContext:\n{context}\n\nQuestion: {question}\n\nAnswer:",
        );
        Self {
            llm,
            vectorstore,
            condense_question_prompt,
            combine_docs_prompt,
            k: 4,
            verbose: false,
        }
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    async fn condense_question(&self, question: &str, chat_history: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("chat_history".to_string(), chat_history.to_string());
        kwargs.insert("question".to_string(), question.to_string());
        let prompt = self.condense_question_prompt.format(&kwargs)?;

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&prompt).into()], None, None)
            .await?;

        Ok(response.content)
    }

    async fn retrieve_docs(&self, query: &str) -> Result<Vec<Document>> {
        self.vectorstore.similarity_search(query, self.k).await
    }
}

#[async_trait]
impl Chain for ChatVectorDBChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["question".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["answer".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if self.verbose {
            info!("ChatVectorDBChain question: {}", question);
        }

        let chat_history = inputs
            .get("chat_history")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let condensed = if chat_history.is_empty() {
            question.clone()
        } else {
            self.condense_question(&question, &chat_history).await?
        };

        let docs = self.retrieve_docs(&condensed).await?;

        let context: String = docs
            .iter()
            .enumerate()
            .map(|(i, d)| format!("[{}] {}", i + 1, d.page_content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let mut format_kwargs = HashMap::new();
        format_kwargs.insert("context".to_string(), context);
        format_kwargs.insert("question".to_string(), question);
        let formatted_prompt = self.combine_docs_prompt.format(&format_kwargs)?;

        let response = self
            .llm
            .predict_messages(&[HumanMessage::new(&formatted_prompt).into()], None, None)
            .await?;

        let mut result = HashMap::new();
        result.insert("answer".to_string(), Value::String(response.content));
        Ok(result)
    }
}
