//! Motorhead memory backend for persistent conversation memory.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::errors::ChainError;
use langchain_core::messages::BaseMessage;
use langchain_core::messages::MessageType;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::traits::BaseMemory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorheadMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MotorheadMemoryResponse {
    messages: Vec<MotorheadMessage>,
    #[serde(default)]
    window_memory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MotorheadAddRequest {
    messages: Vec<MotorheadMessage>,
}

pub struct MotorheadMemory {
    client: reqwest::Client,
    base_url: String,
    session_id: String,
    chat_history: Arc<RwLock<Vec<BaseMessage>>>,
    return_messages: bool,
    memory_key: String,
    input_key: String,
    output_key: String,
    human_prefix: String,
    ai_prefix: String,
}

impl MotorheadMemory {
    pub fn new(base_url: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            session_id: session_id.into(),
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

    fn message_to_motorhead(&self, msg: &BaseMessage) -> MotorheadMessage {
        let role = match msg.message_type {
            MessageType::Human => "human",
            MessageType::AI => "ai",
            MessageType::System => "system",
            MessageType::Tool => "tool",
            MessageType::Function => "function",
            _ => "unknown",
        };
        MotorheadMessage {
            role: role.to_string(),
            content: msg.content.clone(),
        }
    }

    fn motorhead_to_message(&self, msg: &MotorheadMessage) -> BaseMessage {
        let message_type = match msg.role.as_str() {
            "human" => MessageType::Human,
            "ai" => MessageType::AI,
            "system" => MessageType::System,
            "tool" => MessageType::Tool,
            "function" => MessageType::Function,
            _ => MessageType::Generic,
        };
        BaseMessage::new(&msg.content, message_type)
    }

    fn memory_url(&self) -> String {
        format!("{}/sessions/{}/memory", self.base_url.trim_end_matches('/'), self.session_id)
    }

    pub async fn fetch_memory(&self) -> Result<Vec<BaseMessage>> {
        let url = self.memory_url();
        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChainError::MemoryError(format!("Motorhead GET failed: {}", e)))?;

        let body = resp
            .json::<MotorheadMemoryResponse>()
            .await
            .map_err(|e| ChainError::MemoryError(format!("Motorhead response parse failed: {}", e)))?;

        Ok(body.messages.iter().map(|m| self.motorhead_to_message(m)).collect())
    }

    pub async fn add_messages(&self, messages: Vec<MotorheadMessage>) -> Result<()> {
        let url = self.memory_url();
        let body = MotorheadAddRequest { messages };
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::MemoryError(format!("Motorhead POST failed: {}", e)))?;
        Ok(())
    }

    pub fn buffer(&self) -> String {
        let history = self.chat_history.read();
        history
            .iter()
            .map(|msg| {
                let prefix = match msg.message_type {
                    MessageType::Human => &self.human_prefix,
                    MessageType::AI => &self.ai_prefix,
                    MessageType::System => "System",
                    _ => "Unknown",
                };
                format!("{}: {}", prefix, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[async_trait]
impl BaseMemory for MotorheadMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let messages = self.fetch_memory().await?;
        {
            let mut history = self.chat_history.write();
            *history = messages;
        }

        let mut result = HashMap::new();
        if self.return_messages {
            let msgs_value: Vec<Value> = self
                .chat_history
                .read()
                .iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            result.insert(self.memory_key.clone(), Value::Array(msgs_value));
        } else {
            result.insert(self.memory_key.clone(), Value::String(self.buffer()));
        }
        Ok(result)
    }

    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
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

        let human_msg = BaseMessage::new(input_str, MessageType::Human);
        let ai_msg = BaseMessage::new(output_str, MessageType::AI);

        {
            let mut history = self.chat_history.write();
            history.push(human_msg.clone());
            history.push(ai_msg.clone());
        }

        let motorhead_msgs = vec![
            self.message_to_motorhead(&human_msg),
            self.message_to_motorhead(&ai_msg),
        ];
        self.add_messages(motorhead_msgs).await?;

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let url = self.memory_url();
        self.client
            .delete(&url)
            .send()
            .await
            .map_err(|e| ChainError::MemoryError(format!("Motorhead DELETE failed: {}", e)))?;

        self.chat_history.write().clear();
        Ok(())
    }
}
