//! DynamoDB-backed memory.

use std::collections::HashMap;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::{BaseMessage, MessageType};
use serde_json::Value;

use crate::traits::BaseMemory;

pub struct DynamoDBMemory {
    session_id: String,
    table_name: String,
    memory_key: String,
    input_key: String,
    output_key: String,
    return_messages: bool,
    local_messages: Vec<BaseMessage>,
}

impl DynamoDBMemory {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            table_name: std::env::var("DYNAMODB_TABLE")
                .unwrap_or_else(|_| "chat_memory".to_string()),
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            return_messages: false,
            local_messages: Vec::new(),
        }
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
}

#[async_trait]
impl BaseMemory for DynamoDBMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();

        if self.return_messages {
            let msgs: Vec<Value> = self.local_messages.iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            result.insert(self.memory_key.clone(), Value::Array(msgs));
        } else {
            let text = self.local_messages.iter()
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

        let table = &self.table_name;

        tracing::warn!(
            "DynamoDBMemory: Would store to DynamoDB table '{}'. Session: {}, input: {}, output: {}. Install the `aws-sdk-dynamodb` crate and uncomment the real implementation.",
            table, self.session_id, input_str, output_str
        );

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        Ok(())
    }
}
