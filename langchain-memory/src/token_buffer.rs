//! Conversation token buffer memory.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::{BaseMessage, MessageType};
use parking_lot::RwLock;
use serde_json::Value;

use crate::traits::BaseMemory;

pub struct ConversationTokenBufferMemory {
    #[allow(dead_code)]
    llm_name: String,
    max_token_limit: usize,
    chat_history: Arc<RwLock<Vec<BaseMessage>>>,
    memory_key: String,
    return_messages: bool,
    input_key: String,
    output_key: String,
}

impl ConversationTokenBufferMemory {
    pub fn new(llm_name: impl Into<String>, max_token_limit: usize) -> Self {
        Self {
            llm_name: llm_name.into(),
            max_token_limit,
            chat_history: Arc::new(RwLock::new(Vec::new())),
            memory_key: "history".to_string(),
            return_messages: false,
            input_key: "input".to_string(),
            output_key: "output".to_string(),
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

    fn token_count_approx(messages: &[BaseMessage]) -> usize {
        messages.iter().map(|m| m.content.chars().count()).sum()
    }

    fn trim_history(&self) {
        let mut history = self.chat_history.write();
        while Self::token_count_approx(&history) > self.max_token_limit && !history.is_empty() {
            history.remove(0);
        }
    }

    fn buffer_as_string(&self) -> String {
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
impl BaseMemory for ConversationTokenBufferMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        self.trim_history();
        let mut result = HashMap::new();

        if self.return_messages {
            let messages = self.chat_history.read().clone();
            let msgs_value: Vec<Value> = messages.iter().map(|m| serde_json::to_value(m).unwrap_or_default()).collect();
            result.insert(self.memory_key.clone(), Value::Array(msgs_value));
        } else {
            result.insert(self.memory_key.clone(), Value::String(self.buffer_as_string()));
        }

        Ok(result)
    }

    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
        let input = inputs.get(&self.input_key).or_else(|| inputs.values().next()).cloned().unwrap_or(Value::String("".to_string()));
        let output = outputs.get(&self.output_key).or_else(|| outputs.values().next()).cloned().unwrap_or(Value::String("".to_string()));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_buffer_new() {
        let mem = ConversationTokenBufferMemory::new("gpt-4", 100);
        assert_eq!(mem.memory_variables(), vec!["history"]);
    }

    #[tokio::test]
    async fn test_token_buffer_save_and_load() {
        let mem = ConversationTokenBufferMemory::new("gpt-4", 500);
        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("Hello".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("Hi there!".into()));

        mem.save_context(&inputs, &outputs).await.unwrap();
        let vars = mem.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap().as_str().unwrap().to_string();
        assert!(history.contains("Human: Hello"));
        assert!(history.contains("AI: Hi there!"));
    }

    #[tokio::test]
    async fn test_token_buffer_with_return_messages() {
        let mem = ConversationTokenBufferMemory::new("gpt-4", 500)
            .with_return_messages(true);
        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("Hello".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("Hi".into()));

        mem.save_context(&inputs, &outputs).await.unwrap();
        let vars = mem.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap();
        assert!(history.is_array());
        assert_eq!(history.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_token_buffer_with_custom_keys() {
        let mem = ConversationTokenBufferMemory::new("gpt-4", 500)
            .with_input_key("question")
            .with_output_key("answer")
            .with_memory_key("chat");
        assert_eq!(mem.memory_variables(), vec!["chat"]);

        let mut inputs = HashMap::new();
        inputs.insert("question".into(), Value::String("What?".into()));
        let mut outputs = HashMap::new();
        outputs.insert("answer".into(), Value::String("42".into()));

        mem.save_context(&inputs, &outputs).await.unwrap();
        let vars = mem.load_memory_variables(&HashMap::new()).await.unwrap();
        assert!(vars.contains_key("chat"));
    }

    #[tokio::test]
    async fn test_token_buffer_clear() {
        let mem = ConversationTokenBufferMemory::new("gpt-4", 500);
        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("Hi".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("Hello".into()));

        mem.save_context(&inputs, &outputs).await.unwrap();
        mem.clear().await.unwrap();
        let vars = mem.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap().as_str().unwrap();
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_token_buffer_trims() {
        let mem = ConversationTokenBufferMemory::new("gpt-4", 5);
        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("hello world".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("hi".into()));

        mem.save_context(&inputs, &outputs).await.unwrap();
        let vars = mem.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap().as_str().unwrap();
        // Should have trimmed to fit within 5 chars
        assert!(!history.contains("Human: hello world") || history.len() < 20);
    }

    #[test]
    fn test_token_count_approx() {
        let msgs = vec![
            BaseMessage::new("hello", MessageType::Human),
            BaseMessage::new("world", MessageType::AI),
        ];
        assert_eq!(ConversationTokenBufferMemory::token_count_approx(&msgs), 10);
    }

    #[test]
    fn test_token_count_approx_empty() {
        assert_eq!(ConversationTokenBufferMemory::token_count_approx(&[]), 0);
    }
}
