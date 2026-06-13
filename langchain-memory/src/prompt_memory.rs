//! Prompt memory that combines variables from multiple memory stores.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use serde_json::Value;

use crate::traits::BaseMemory;

pub struct PromptMemory {
    memories: HashMap<String, Arc<dyn BaseMemory>>,
    prompt_template: String,
}

impl PromptMemory {
    pub fn new(memories: HashMap<String, Arc<dyn BaseMemory>>, prompt_template: impl Into<String>) -> Self {
        Self {
            memories,
            prompt_template: prompt_template.into(),
        }
    }

    pub fn with_memory(mut self, key: impl Into<String>, memory: Arc<dyn BaseMemory>) -> Self {
        self.memories.insert(key.into(), memory);
        self
    }

    pub fn with_prompt_template(mut self, template: impl Into<String>) -> Self {
        self.prompt_template = template.into();
        self
    }
}

#[async_trait]
impl BaseMemory for PromptMemory {
    fn memory_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        for m in self.memories.values() {
            vars.extend(m.memory_variables());
        }
        vars
    }

    async fn load_memory_variables(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();
        for m in self.memories.values() {
            let vars = m.load_memory_variables(inputs).await?;
            result.extend(vars);
        }
        Ok(result)
    }

    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
        for m in self.memories.values() {
            m.save_context(inputs, outputs).await?;
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        for m in self.memories.values() {
            m.clear().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_buffer::ConversationTokenBufferMemory;

    fn make_memory(key: &str) -> (String, Arc<dyn BaseMemory>) {
        (key.into(), Arc::new(ConversationTokenBufferMemory::new("test", 1000)))
    }

    #[tokio::test]
    async fn test_prompt_memory_empty() {
        let pm = PromptMemory::new(HashMap::new(), "{history}");
        assert!(pm.memory_variables().is_empty());
    }

    #[tokio::test]
    async fn test_prompt_memory_single_memory() {
        let (k, m) = make_memory("chat");
        let mut memories = HashMap::new();
        memories.insert(k, m);
        let pm = PromptMemory::new(memories, "{history}");
        assert_eq!(pm.memory_variables(), vec!["history"]);
    }

    #[tokio::test]
    async fn test_prompt_memory_with_builder() {
        let (k, m) = make_memory("history");
        let pm = PromptMemory::new(HashMap::new(), "")
            .with_memory(k, m)
            .with_prompt_template("{history}");
        assert_eq!(pm.memory_variables(), vec!["history"]);
    }

    #[tokio::test]
    async fn test_prompt_memory_save_and_load() {
        let (k, m) = make_memory("chat");
        let mut memories = HashMap::new();
        memories.insert(k, m);
        let pm = PromptMemory::new(memories, "{history}");

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("Hi".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("Hello".into()));

        pm.save_context(&inputs, &outputs).await.unwrap();
        let vars = pm.load_memory_variables(&HashMap::new()).await.unwrap();
        assert!(vars.contains_key("history"));
    }

    #[tokio::test]
    async fn test_prompt_memory_clear() {
        let (k, m) = make_memory("chat");
        let mut memories = HashMap::new();
        memories.insert(k, m);
        let pm = PromptMemory::new(memories, "{history}");

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("Hi".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("Hello".into()));

        pm.save_context(&inputs, &outputs).await.unwrap();
        pm.clear().await.unwrap();
        let vars = pm.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap().as_str().unwrap();
        assert!(history.is_empty());
    }
}
