//! Combined memory that aggregates multiple memory stores.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use serde_json::Value;

use crate::traits::BaseMemory;

pub struct CombinedMemory {
    memories: Vec<Arc<dyn BaseMemory>>,
}

impl CombinedMemory {
    pub fn new(memories: Vec<Arc<dyn BaseMemory>>) -> Self {
        Self { memories }
    }

    pub fn add_memory(&mut self, memory: Arc<dyn BaseMemory>) {
        self.memories.push(memory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_buffer::ConversationTokenBufferMemory;

    fn make_memory() -> Arc<dyn BaseMemory> {
        Arc::new(ConversationTokenBufferMemory::new("test", 1000))
    }

    #[tokio::test]
    async fn test_combined_memory_empty() {
        let cm = CombinedMemory::new(vec![]);
        assert!(cm.memory_variables().is_empty());
    }

    #[tokio::test]
    async fn test_combined_memory_single() {
        let mem = make_memory();
        let cm = CombinedMemory::new(vec![mem]);
        assert_eq!(cm.memory_variables(), vec!["history"]);
    }

    #[tokio::test]
    async fn test_combined_memory_multiple() {
        let mem1 = make_memory();
        let mem2 = make_memory();
        let mut cm = CombinedMemory::new(vec![mem1]);
        cm.add_memory(mem2);
        assert_eq!(cm.memory_variables(), vec!["history", "history"]);
    }

    #[tokio::test]
    async fn test_combined_memory_save_and_load() {
        let mem = make_memory();
        let cm = CombinedMemory::new(vec![mem]);

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("hi".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("hello".into()));

        cm.save_context(&inputs, &outputs).await.unwrap();
        let vars = cm.load_memory_variables(&HashMap::new()).await.unwrap();
        assert!(vars.contains_key("history"));
    }

    #[tokio::test]
    async fn test_combined_memory_clear() {
        let mem = make_memory();
        let cm = CombinedMemory::new(vec![mem]);

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("hi".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("hello".into()));

        cm.save_context(&inputs, &outputs).await.unwrap();
        cm.clear().await.unwrap();
        let vars = cm.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap();
        if let Value::String(s) = history {
            assert!(s.is_empty());
        }
    }
}

#[async_trait]
impl BaseMemory for CombinedMemory {
    fn memory_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        for m in &self.memories {
            vars.extend(m.memory_variables());
        }
        vars
    }

    async fn load_memory_variables(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();
        for m in &self.memories {
            let vars = m.load_memory_variables(inputs).await?;
            result.extend(vars);
        }
        Ok(result)
    }

    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
        for m in &self.memories {
            m.save_context(inputs, outputs).await?;
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        for m in &self.memories {
            m.clear().await?;
        }
        Ok(())
    }
}
