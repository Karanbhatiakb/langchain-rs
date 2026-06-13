//! Redis-backed memory.

use std::collections::HashMap;
use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use langchain_core::messages::{BaseMessage, MessageType};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde_json::Value;

use crate::traits::BaseMemory;

pub struct RedisChatMemory {
    conn: ConnectionManager,
    session_id: String,
    memory_key: String,
    input_key: String,
    output_key: String,
    return_messages: bool,
    ttl_seconds: Option<u64>,
}

impl RedisChatMemory {
    pub async fn new(
        redis_url: &str,
        session_id: impl Into<String>,
    ) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| ChainError::MemoryError(format!("Failed to create Redis client: {}", e)))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            conn,
            session_id: session_id.into(),
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            return_messages: false,
            ttl_seconds: None,
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

    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }

    #[allow(dead_code)]
    fn messages_key(&self) -> String {
        format!("{}:messages", self.session_id)
    }

    fn list_key(&self) -> String {
        format!("{}:list", self.session_id)
    }

    async fn store_message(&self, role: &str, content: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        let msg = serde_json::json!({"role": role, "content": content});
        let msg_str = msg.to_string();
        let list_key = self.list_key();

        conn.rpush::<_, _, ()>(&list_key, &msg_str)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to store message in Redis: {}", e)))?;

        if let Some(ttl) = self.ttl_seconds {
            let _: () = conn.expire(&list_key, ttl as i64)
                .await
                .map_err(|e| ChainError::MemoryError(format!("Failed to set TTL: {}", e)))?;
        }

        Ok(())
    }

    async fn get_messages(&self) -> Result<Vec<BaseMessage>> {
        let mut conn = self.conn.clone();
        let list_key = self.list_key();

        let msg_strs: Vec<String> = conn.lrange(&list_key, 0, -1)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to read messages from Redis: {}", e)))?;

        let messages = msg_strs.iter()
            .filter_map(|s| {
                let v: serde_json::Value = serde_json::from_str(s).ok()?;
                let role = v.get("role")?.as_str()?;
                let content = v.get("content")?.as_str()?;
                let msg_type = match role {
                    "human" => MessageType::Human,
                    "ai" => MessageType::AI,
                    "system" => MessageType::System,
                    _ => MessageType::Generic,
                };
                Some(BaseMessage::new(content.to_string(), msg_type))
            })
            .collect();

        Ok(messages)
    }
}

#[async_trait]
impl BaseMemory for RedisChatMemory {
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
        let mut conn = self.conn.clone();
        let list_key = self.list_key();

        conn.del::<_, ()>(&list_key)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to clear Redis messages: {}", e)))?;

        Ok(())
    }
}
