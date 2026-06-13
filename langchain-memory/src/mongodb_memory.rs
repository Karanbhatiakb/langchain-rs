//! MongoDB-backed memory.

use std::collections::HashMap;
use async_trait::async_trait;
use futures::StreamExt;
use langchain_core::errors::{ChainError, Result};
use langchain_core::messages::{BaseMessage, MessageType};
use mongodb::bson::{doc, Document as BsonDocument};
use mongodb::options::ClientOptions;
use mongodb::{Client, Collection};
use serde_json::Value;

use crate::traits::BaseMemory;

pub struct MongoDBChatMemory {
    collection: Collection<BsonDocument>,
    session_id: String,
    memory_key: String,
    input_key: String,
    output_key: String,
    return_messages: bool,
}

impl MongoDBChatMemory {
    pub async fn new(
        connection_string: &str,
        database_name: &str,
        collection_name: &str,
        session_id: impl Into<String>,
    ) -> Result<Self> {
        let client_options = ClientOptions::parse(connection_string)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to parse MongoDB connection string: {}", e)))?;

        let client = Client::with_options(client_options)
            .map_err(|e| ChainError::MemoryError(format!("Failed to create MongoDB client: {}", e)))?;

        let collection = client.database(database_name).collection(collection_name);

        Ok(Self {
            collection,
            session_id: session_id.into(),
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            return_messages: false,
        })
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

    pub fn with_return_messages(mut self, value: bool) -> Self {
        self.return_messages = value;
        self
    }

    async fn store_message(&self, role: &str, content: &str) -> Result<()> {
        let bson_doc = doc! {
            "session_id": &self.session_id,
            "role": role,
            "content": content,
            "created_at": chrono::Utc::now().to_rfc3339(),
        };

        self.collection.insert_one(bson_doc)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to insert message into MongoDB: {}", e)))?;

        Ok(())
    }

    async fn get_messages(&self) -> Result<Vec<BaseMessage>> {
        let filter = doc! { "session_id": &self.session_id };
        let mut cursor = self.collection.find(filter)
            .sort(doc! { "created_at": 1 })
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to query MongoDB: {}", e)))?;

        let mut messages = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(bson_doc) => {
                    let role = bson_doc.get_str("role").unwrap_or("");
                    let content = bson_doc.get_str("content").unwrap_or("");
                    let msg_type = match role {
                        "human" => MessageType::Human,
                        "ai" => MessageType::AI,
                        "system" => MessageType::System,
                        _ => MessageType::Generic,
                    };
                    messages.push(BaseMessage::new(content.to_string(), msg_type));
                }
                Err(e) => {
                    return Err(ChainError::MemoryError(format!("Failed to read MongoDB document: {}", e)));
                }
            }
        }

        Ok(messages)
    }
}

#[async_trait]
impl BaseMemory for MongoDBChatMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();
        let messages = self.get_messages().await?;

        if self.return_messages {
            let msgs: Vec<Value> = messages.iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            result.insert(self.memory_key.clone(), Value::Array(msgs));
        } else {
            let text = messages.iter()
                .map(|m| {
                    let prefix = match m.message_type {
                        MessageType::Human => "Human",
                        MessageType::AI => "AI",
                        _ => "System",
                    };
                    format!("{}: {}", prefix, m.content)
                })
                .collect::<Vec<_>>()
                .join("\n");
            result.insert(self.memory_key.clone(), Value::String(text));
        }

        Ok(result)
    }

    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
        let input = inputs.get(&self.input_key)
            .or_else(|| inputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));

        let output = outputs.get(&self.output_key)
            .or_else(|| outputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));

        let input_str = input.as_str().unwrap_or("").to_string();
        let output_str = output.as_str().unwrap_or("").to_string();

        self.store_message("human", &input_str).await?;
        self.store_message("ai", &output_str).await?;

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let filter = doc! { "session_id": &self.session_id };
        self.collection.delete_many(filter)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to clear MongoDB messages: {}", e)))?;

        Ok(())
    }
}
