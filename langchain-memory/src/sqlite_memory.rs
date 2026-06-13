//! SQLite-backed memory.

use std::collections::HashMap;
use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use langchain_core::messages::{BaseMessage, MessageType};
use serde_json::Value;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use crate::traits::BaseMemory;

pub struct SQLiteChatMemory {
    pool: SqlitePool,
    session_id: String,
    memory_key: String,
    input_key: String,
    output_key: String,
    return_messages: bool,
    table_name: String,
}

impl SQLiteChatMemory {
    pub async fn new(
        database_path: &str,
        session_id: impl Into<String>,
    ) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_path)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to connect to SQLite: {}", e)))?;

        let instance = Self {
            pool,
            session_id: session_id.into(),
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            return_messages: false,
            table_name: "chat_history".to_string(),
        };

        instance.init_table().await?;
        Ok(instance)
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

    pub fn with_table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = name.into();
        self
    }

    async fn init_table(&self) -> Result<()> {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT DEFAULT (datetime('now'))
            )",
            self.table_name
        );
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to create table: {}", e)))?;
        Ok(())
    }

    async fn get_messages(&self) -> Result<Vec<BaseMessage>> {
        let query = format!(
            "SELECT role, content FROM {} WHERE session_id = ?1 ORDER BY id ASC",
            self.table_name
        );
        let rows: Vec<(String, String)> = sqlx::query_as(&query)
            .bind(&self.session_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to fetch messages: {}", e)))?;

        Ok(rows.into_iter().map(|(role, content)| {
            let msg_type = match role.as_str() {
                "human" => MessageType::Human,
                "ai" => MessageType::AI,
                "system" => MessageType::System,
                _ => MessageType::Generic,
            };
            BaseMessage::new(content, msg_type)
        }).collect())
    }
}

#[async_trait]
impl BaseMemory for SQLiteChatMemory {
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

        let query = format!(
            "INSERT INTO {} (session_id, role, content) VALUES (?1, ?2, ?3)",
            self.table_name
        );

        sqlx::query(&query)
            .bind(&self.session_id)
            .bind("human")
            .bind(&input_str)
            .execute(&self.pool)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to insert message: {}", e)))?;

        sqlx::query(&query)
            .bind(&self.session_id)
            .bind("ai")
            .bind(&output_str)
            .execute(&self.pool)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to insert message: {}", e)))?;

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let query = format!(
            "DELETE FROM {} WHERE session_id = ?1",
            self.table_name
        );
        sqlx::query(&query)
            .bind(&self.session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChainError::MemoryError(format!("Failed to clear messages: {}", e)))?;
        Ok(())
    }
}
