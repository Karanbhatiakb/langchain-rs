//! Conversation summary memory.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::{BaseMessage, MessageType};
use parking_lot::RwLock;
use serde_json::Value;
use langchain_llms::traits::BaseLLM;

use crate::traits::BaseMemory;

pub struct ConversationSummaryMemory {
    llm: Arc<dyn BaseLLM>,
    chat_history: Arc<RwLock<Vec<BaseMessage>>>,
    summary: Arc<RwLock<String>>,
    max_token_limit: usize,
    return_messages: bool,
    memory_key: String,
    input_key: String,
    output_key: String,
    human_prefix: String,
    ai_prefix: String,
    prompt_template: String,
}

impl ConversationSummaryMemory {
    pub fn new(llm: Arc<dyn BaseLLM>) -> Self {
        Self {
            llm,
            chat_history: Arc::new(RwLock::new(Vec::new())),
            summary: Arc::new(RwLock::new(String::new())),
            max_token_limit: 2000,
            return_messages: false,
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            human_prefix: "Human".to_string(),
            ai_prefix: "AI".to_string(),
            prompt_template: "Progressively summarize the lines of conversation provided, adding onto the previous summary returning a new summary.\n\nCurrent summary:\n{summary}\n\nNew lines of conversation:\n{new_lines}\n\nNew summary:".to_string(),
        }
    }

    pub fn with_max_token_limit(mut self, limit: usize) -> Self {
        self.max_token_limit = limit;
        self
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

    pub fn summary(&self) -> String {
        self.summary.read().clone()
    }

    pub fn buffer(&self) -> String {
        let history = self.chat_history.read();
        history.iter()
            .map(|msg| {
                let prefix = match msg.message_type {
                    MessageType::Human => &self.human_prefix,
                    MessageType::AI => &self.ai_prefix,
                    _ => "System",
                };
                format!("{}: {}", prefix, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    async fn summarize(&self) -> Result<String> {
        let summary = self.summary.read().clone();
        let new_lines = self.buffer();
        let prompt = self.prompt_template
            .replace("{summary}", &summary)
            .replace("{new_lines}", &new_lines);

        let result = self.llm.generate(&[prompt], None).await?;
        let new_summary = result.generations.first()
            .and_then(|g| g.first())
            .map(|g| g.text.clone())
            .unwrap_or_default();

        Ok(new_summary)
    }

    pub async fn predict_new_summary(&self, messages: &[BaseMessage]) -> Result<String> {
        let new_lines = messages.iter()
            .map(|msg| {
                let prefix = match msg.message_type {
                    MessageType::Human => &self.human_prefix,
                    MessageType::AI => &self.ai_prefix,
                    _ => "System",
                };
                format!("{}: {}", prefix, msg.content)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let summary = self.summary.read().clone();
        let prompt = self.prompt_template
            .replace("{summary}", &summary)
            .replace("{new_lines}", &new_lines);

        let result = self.llm.generate(&[prompt], None).await?;
        Ok(result.generations.first()
            .and_then(|g| g.first())
            .map(|g| g.text.clone())
            .unwrap_or_default())
    }

    fn estimate_tokens(text: &str) -> usize {
        text.len() / 4 + 1
    }
}

#[async_trait]
impl BaseMemory for ConversationSummaryMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();
        let summary = self.summary.read().clone();

        if self.return_messages {
            let msg = BaseMessage::new(summary, MessageType::System);
            result.insert(self.memory_key.clone(), serde_json::json!([msg]));
        } else {
            result.insert(self.memory_key.clone(), Value::String(summary));
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

        let buffer_text = self.buffer();
        let current_summary = self.summary.read().clone();
        let total_tokens = Self::estimate_tokens(&current_summary) + Self::estimate_tokens(&buffer_text);

        if total_tokens > self.max_token_limit {
            let new_summary = self.summarize().await?;
            *self.summary.write() = new_summary;
            self.chat_history.write().clear();
        }

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.chat_history.write().clear();
        *self.summary.write() = String::new();
        Ok(())
    }
}
