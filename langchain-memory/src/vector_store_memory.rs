//! Vector store-backed memory — retrieves relevant conversation snippets.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use langchain_core::messages::{BaseMessage, MessageType};
use langchain_embeddings::traits::Embeddings;
use langchain_vectorstores::traits::VectorStore;
use serde_json::Value;
use parking_lot::RwLock;

use crate::traits::BaseMemory;

/// Memory that stores conversation history in a vector store for semantic
/// retrieval of relevant past context.
pub struct VectorStoreRetrieverMemory {
    vectorstore: Arc<dyn VectorStore>,
    embeddings: Arc<dyn Embeddings>,
    chat_history: Arc<RwLock<Vec<BaseMessage>>>,
    return_messages: bool,
    memory_key: String,
    input_key: String,
    output_key: String,
    k: usize,
    relevance_score: f64,
}

impl VectorStoreRetrieverMemory {
    pub fn new(vectorstore: Arc<dyn VectorStore>, embeddings: Arc<dyn Embeddings>) -> Self {
        Self {
            vectorstore,
            embeddings,
            chat_history: Arc::new(RwLock::new(Vec::new())),
            return_messages: false,
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            k: 5,
            relevance_score: 0.7,
        }
    }

    pub fn with_return_messages(mut self, value: bool) -> Self {
        self.return_messages = value;
        self
    }

    pub fn with_memory_key(mut self, key: impl Into<String>) -> Self {
        self.memory_key = key.into();
        self
    }

    pub fn with_input_key(mut self, key: impl Into<String>) -> Self {
        self.input_key = key.into();
        self
    }

    pub fn with_output_key(mut self, key: impl Into<String>) -> Self {
        self.output_key = key.into();
        self
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn with_relevance_score(mut self, score: f64) -> Self {
        self.relevance_score = score;
        self
    }

    fn conversation_buffer(&self) -> String {
        let history = self.chat_history.read();
        history
            .iter()
            .map(|msg| {
                let prefix = match msg.message_type {
                    MessageType::Human => "Human",
                    MessageType::AI => "AI",
                    _ => "System",
                };
                format!("{}: {}", prefix, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[async_trait]
impl BaseMemory for VectorStoreRetrieverMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(
        &self,
        inputs: &HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>> {
        let input = inputs
            .get(&self.input_key)
            .or_else(|| inputs.values().next())
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut result = HashMap::new();
        if input.is_empty() {
            if self.return_messages {
                let msgs: Vec<Value> = self
                    .chat_history
                    .read()
                    .iter()
                    .map(|m| serde_json::to_value(m).unwrap_or_default())
                    .collect();
                result.insert(self.memory_key.clone(), Value::Array(msgs));
            } else {
                result.insert(
                    self.memory_key.clone(),
                    Value::String(self.conversation_buffer()),
                );
            }
            return Ok(result);
        }
        let query_embedding = self.embeddings.embed_query(input).await?;
        let similar_docs = self
            .vectorstore
            .similarity_search_by_vector(query_embedding, self.k)
            .await?;
        let memory_text = similar_docs
            .iter()
            .map(|d| d.page_content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if self.return_messages {
            let msg = BaseMessage::new(memory_text, MessageType::System);
            result.insert(self.memory_key.clone(), serde_json::json!([msg]));
        } else {
            result.insert(self.memory_key.clone(), Value::String(memory_text));
        }
        Ok(result)
    }

    async fn save_context(
        &self,
        inputs: &HashMap<String, Value>,
        outputs: &HashMap<String, Value>,
    ) -> Result<()> {
        let input = inputs
            .get(&self.input_key)
            .or_else(|| inputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));
        let output = outputs
            .get(&self.output_key)
            .or_else(|| outputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));
        let input_str = input.as_str().unwrap_or("").to_string();
        let output_str = output.as_str().unwrap_or("").to_string();
        {
            let mut history = self.chat_history.write();
            history.push(BaseMessage::new(input_str.clone(), MessageType::Human));
            history.push(BaseMessage::new(output_str.clone(), MessageType::AI));
        }
        let combined = format!("Human: {}\nAI: {}", input_str, output_str);
        let docs = vec![Document::new(combined)];
        self.vectorstore.add_documents(docs).await?;
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.chat_history.write().clear();
        Ok(())
    }
}
