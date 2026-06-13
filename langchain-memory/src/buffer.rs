//! Conversation buffer memory.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;
use serde_json::Value;

use langchain_core::messages::MessageType;

use crate::traits::BaseMemory;

pub struct ConversationBufferMemory {
    chat_history: Arc<RwLock<Vec<BaseMessage>>>,
    return_messages: bool,
    memory_key: String,
    input_key: String,
    output_key: String,
    human_prefix: String,
    ai_prefix: String,
}

impl ConversationBufferMemory {
    pub fn new() -> Self {
        Self {
            chat_history: Arc::new(RwLock::new(Vec::new())),
            return_messages: false,
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            human_prefix: "Human".to_string(),
            ai_prefix: "AI".to_string(),
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

    pub fn with_human_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.human_prefix = prefix.into();
        self
    }

    pub fn with_ai_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.ai_prefix = prefix.into();
        self
    }

    pub fn chat_history(&self) -> Vec<BaseMessage> {
        self.chat_history.read().clone()
    }

    pub fn messages(&self) -> Vec<BaseMessage> {
        self.chat_history.read().clone()
    }

    pub fn add_message(&self, message: BaseMessage) {
        self.chat_history.write().push(message);
    }

    pub fn get_messages(&self) -> Vec<BaseMessage> {
        self.chat_history.read().clone()
    }

    pub fn buffer(&self) -> String {
        let history = self.chat_history.read();
        history.iter()
            .map(|msg| {
                let prefix = match msg.message_type {
                    langchain_core::messages::MessageType::Human => &self.human_prefix,
                    langchain_core::messages::MessageType::AI => &self.ai_prefix,
                    langchain_core::messages::MessageType::System => "System",
                    _ => "Unknown",
                };
                format!("{}: {}", prefix, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for ConversationBufferMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseMemory for ConversationBufferMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();

        if self.return_messages {
            let messages = self.chat_history.read().clone();
            let msgs_value: Vec<Value> = messages.iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            result.insert(self.memory_key.clone(), Value::Array(msgs_value));
        } else {
            result.insert(self.memory_key.clone(), Value::String(self.buffer()));
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

        {
            let mut history = self.chat_history.write();
            history.push(BaseMessage::new(input_str, MessageType::Human));
            history.push(BaseMessage::new(output_str, MessageType::AI));
        }

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.chat_history.write().clear();
        Ok(())
    }
}
