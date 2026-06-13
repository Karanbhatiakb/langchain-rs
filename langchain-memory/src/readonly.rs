//! Read-only memory wrapper.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use serde_json::Value;

use crate::traits::BaseMemory;

pub struct ReadOnlyMemory {
    memory: Arc<dyn BaseMemory>,
}

impl ReadOnlyMemory {
    pub fn new(memory: Arc<dyn BaseMemory>) -> Self {
        Self { memory }
    }
}

#[async_trait]
impl BaseMemory for ReadOnlyMemory {
    fn memory_variables(&self) -> Vec<String> {
        self.memory.memory_variables()
    }

    async fn load_memory_variables(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        self.memory.load_memory_variables(inputs).await
    }

    async fn save_context(&self, _inputs: &HashMap<String, Value>, _outputs: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        Ok(())
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
    async fn test_readonly_delegates_variables() {
        let inner = make_memory();
        let ro = ReadOnlyMemory::new(inner);
        assert_eq!(ro.memory_variables(), vec!["history"]);
    }

    #[tokio::test]
    async fn test_readonly_does_not_save() {
        let inner = make_memory();
        let ro = ReadOnlyMemory::new(inner.clone());

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("hi".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("hello".into()));

        // Save via read-only should be no-op
        ro.save_context(&inputs, &outputs).await.unwrap();

        // Inner memory should not have been updated
        let vars = inner.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap().as_str().unwrap();
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_readonly_loads_from_inner() {
        let inner = make_memory();
        let ro = ReadOnlyMemory::new(inner.clone());

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("hi".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("hello".into()));

        // Save to inner directly
        inner.save_context(&inputs, &outputs).await.unwrap();

        // Read-only should reflect inner state
        let vars = ro.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap().as_str().unwrap();
        assert!(history.contains("Human: hi"));
    }

    #[tokio::test]
    async fn test_readonly_clear_is_noop() {
        let inner = make_memory();
        let ro = ReadOnlyMemory::new(inner.clone());

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("hi".into()));
        let mut outputs = HashMap::new();
        outputs.insert("output".into(), Value::String("hello".into()));

        inner.save_context(&inputs, &outputs).await.unwrap();
        ro.clear().await.unwrap();

        // Inner should still have data after read-only clear
        let vars = inner.load_memory_variables(&HashMap::new()).await.unwrap();
        let history = vars.get("history").unwrap().as_str().unwrap();
        assert!(history.contains("Human: hi"));
    }
}
