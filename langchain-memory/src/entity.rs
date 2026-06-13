//! Conversation entity memory.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use langchain_core::errors::Result;
use langchain_core::messages::{BaseMessage, MessageType};
use parking_lot::RwLock;
use serde_json::Value;
use langchain_llms::traits::BaseLLM;

use crate::traits::BaseMemory;

#[derive(Debug, Clone)]
pub struct EntitySummary {
    pub name: String,
    pub summary: String,
    pub mentions: Vec<String>,
}

pub struct EntityMemory {
    llm: Arc<dyn BaseLLM>,
    chat_history: Arc<RwLock<Vec<BaseMessage>>>,
    entities: Arc<DashMap<String, EntitySummary>>,
    return_messages: bool,
    memory_key: String,
    input_key: String,
    output_key: String,
    human_prefix: String,
    ai_prefix: String,
    entity_extraction_prompt: String,
    entity_summarization_prompt: String,
}

impl EntityMemory {
    pub fn new(llm: Arc<dyn BaseLLM>) -> Self {
        Self {
            llm,
            chat_history: Arc::new(RwLock::new(Vec::new())),
            entities: Arc::new(DashMap::new()),
            return_messages: false,
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            human_prefix: "Human".to_string(),
            ai_prefix: "AI".to_string(),
            entity_extraction_prompt: "Identify and extract all named entities from the following conversation. Return them as a comma-separated list.\n\nConversation:\n{conversation}\n\nEntities:".to_string(),
            entity_summarization_prompt: "Update the summary of the entity '{entity}' based on the new information provided. Keep it concise.\n\nCurrent summary:\n{current_summary}\n\nNew information:\n{information}\n\nUpdated summary:".to_string(),
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

    pub fn with_entity_extraction_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.entity_extraction_prompt = prompt.into();
        self
    }

    pub fn with_entity_summarization_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.entity_summarization_prompt = prompt.into();
        self
    }

    pub fn entities(&self) -> Vec<(String, String)> {
        self.entities.iter()
            .map(|e| (e.name.clone(), e.summary.clone()))
            .collect()
    }

    pub fn entity_store(&self) -> Arc<DashMap<String, EntitySummary>> {
        self.entities.clone()
    }

    #[allow(dead_code)]
    fn conversation_buffer(&self) -> String {
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

    async fn extract_entities(&self, conversation: &str) -> Result<Vec<String>> {
        let prompt = self.entity_extraction_prompt.replace("{conversation}", conversation);
        let result = self.llm.generate(&[prompt], None).await?;
        let text = result.generations.first()
            .and_then(|g| g.first())
            .map(|g| g.text.clone())
            .unwrap_or_default();

        Ok(text.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    async fn summarize_entity(&self, entity: &str, current_summary: &str, information: &str) -> Result<String> {
        let prompt = self.entity_summarization_prompt
            .replace("{entity}", entity)
            .replace("{current_summary}", current_summary)
            .replace("{information}", information);

        let result = self.llm.generate(&[prompt], None).await?;
        Ok(result.generations.first()
            .and_then(|g| g.first())
            .map(|g| g.text.clone())
            .unwrap_or_default())
    }
}

#[async_trait]
impl BaseMemory for EntityMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();
        let entities_info: HashMap<String, Value> = self.entities.iter()
            .map(|e| (e.name.clone(), Value::String(e.summary.clone())))
            .collect();
        result.insert(self.memory_key.clone(), serde_json::to_value(&entities_info).unwrap_or_default());
        Ok(result)
    }

    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
        let input_val = inputs.get(&self.input_key)
            .or_else(|| inputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));

        let output_val = outputs.get(&self.output_key)
            .or_else(|| outputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));

        let input_str = input_val.as_str().unwrap_or("").to_string();
        let output_str = output_val.as_str().unwrap_or("").to_string();

        {
            let mut history = self.chat_history.write();
            history.push(BaseMessage::new(input_str.clone(), MessageType::Human));
            history.push(BaseMessage::new(output_str.clone(), MessageType::AI));
        }

        let conversation = format!("{}: {}\n{}: {}", self.human_prefix, input_str, self.ai_prefix, output_str);
        let entity_names = self.extract_entities(&conversation).await?;

        for entity_name in entity_names {
            let current_summary = self.entities
                .get(&entity_name)
                .map(|e| e.summary.clone())
                .unwrap_or_default();

            let new_summary = self.summarize_entity(&entity_name, &current_summary, &conversation).await?;

            self.entities.insert(entity_name.clone(), EntitySummary {
                name: entity_name.clone(),
                summary: new_summary,
                mentions: vec![conversation.clone()],
            });
        }

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.chat_history.write().clear();
        self.entities.clear();
        Ok(())
    }
}
