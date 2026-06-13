//! Conversation window buffer memory.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;
use serde_json::Value;

use langchain_core::messages::MessageType;

use crate::traits::BaseMemory;

pub struct ConversationBufferWindowMemory {
    chat_history: Arc<RwLock<Vec<BaseMessage>>>,
    k: usize,
    return_messages: bool,
    memory_key: String,
    input_key: String,
    output_key: String,
    human_prefix: String,
    ai_prefix: String,
}

impl ConversationBufferWindowMemory {
    pub fn new(k: usize) -> Self {
        Self {
            chat_history: Arc::new(RwLock::new(Vec::new())),
            k,
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

    fn trim_history(&self) {
        let mut history = self.chat_history.write();
        while history.len() > self.k * 2 {
            history.remove(0);
        }
    }

    fn buffer(&self) -> String {
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

#[async_trait]
impl BaseMemory for ConversationBufferWindowMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();

        if self.return_messages {
            let history = self.chat_history.read();
            let start = if history.len() > self.k * 2 { history.len() - self.k * 2 } else { 0 };
            let msgs: Vec<Value> = history[start..].iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            result.insert(self.memory_key.clone(), Value::Array(msgs));
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

        self.trim_history();
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.chat_history.write().clear();
        Ok(())
    }
}
