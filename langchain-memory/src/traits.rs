//! Core [`BaseMemory`] trait for conversation memory.

use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use langchain_core::errors::Result;

/// Trait for memory stores that maintain conversation context across turns.
///
/// Implementors manage variables (e.g., chat history), load them given
/// current inputs, save context after each turn, and can be cleared.
#[async_trait]
pub trait BaseMemory: Send + Sync {
    /// Returns the variable names this memory manages.
    fn memory_variables(&self) -> Vec<String>;
    /// Loads memory variables relevant to the current inputs.
    async fn load_memory_variables(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>>;
    /// Saves the context of a conversation turn (inputs + outputs).
    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()>;
    /// Clears all stored memory.
    async fn clear(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMemory;

    #[async_trait]
    impl BaseMemory for TestMemory {
        fn memory_variables(&self) -> Vec<String> { vec!["history".into()] }
        async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
            Ok(HashMap::from([("history".into(), Value::String("Hi".into()))]))
        }
        async fn save_context(&self, _inputs: &HashMap<String, Value>, _outputs: &HashMap<String, Value>) -> Result<()> { Ok(()) }
        async fn clear(&self) -> Result<()> { Ok(()) }
    }

    #[tokio::test]
    async fn test_memory_variables() {
        let mem = TestMemory;
        assert_eq!(mem.memory_variables(), vec!["history"]);
    }

    #[tokio::test]
    async fn test_load_memory_variables() {
        let mem = TestMemory;
        let vars = mem.load_memory_variables(&HashMap::new()).await.unwrap();
        assert_eq!(vars.get("history").and_then(|v| v.as_str()), Some("Hi"));
    }

    #[tokio::test]
    async fn test_save_context() {
        let mem = TestMemory;
        let result = mem.save_context(&HashMap::new(), &HashMap::new()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear() {
        let mem = TestMemory;
        let result = mem.clear().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<TestMemory>();
        assert_sync::<TestMemory>();
    }
}
