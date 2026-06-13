use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_memory::traits::BaseMemory;
use parking_lot::RwLock;
use serde_json::Value;

struct TestMemory {
    store: Arc<RwLock<HashMap<String, Value>>>,
}

impl TestMemory {
    fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl BaseMemory for TestMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec!["response".into(), "msg".into()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        Ok(self.store.read().clone())
    }

    async fn save_context(&self, _inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
        let mut store = self.store.write();
        for (k, v) in outputs {
            store.insert(k.clone(), v.clone());
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.store.write().clear();
        Ok(())
    }
}

#[tokio::test]
async fn test_memory_save_and_load() {
    let memory = TestMemory::new();
    let mut outputs = HashMap::new();
    outputs.insert("response".to_string(), Value::String("hello".to_string()));
    memory.save_context(&HashMap::new(), &outputs).await.unwrap();
    let vars = memory.load_memory_variables(&HashMap::new()).await.unwrap();
    assert_eq!(vars.get("response").unwrap(), "hello");
}

#[tokio::test]
async fn test_memory_clear() {
    let memory = TestMemory::new();
    let mut outputs = HashMap::new();
    outputs.insert("key".to_string(), Value::String("val".to_string()));
    memory.save_context(&HashMap::new(), &outputs).await.unwrap();
    memory.clear().await.unwrap();
    let vars = memory.load_memory_variables(&HashMap::new()).await.unwrap();
    assert!(vars.is_empty());
}

#[tokio::test]
async fn test_memory_window_keeps_last_k() {
    let memory = TestMemory::new();
    for i in 0..5 {
        let mut outputs = HashMap::new();
        outputs.insert("msg".to_string(), Value::String(format!("msg_{}", i)));
        memory.save_context(&HashMap::new(), &outputs).await.unwrap();
    }
    let vars = memory.load_memory_variables(&HashMap::new()).await.unwrap();
    let msg = vars.get("msg").unwrap().as_str().unwrap();
    assert_eq!(msg, "msg_4");
}
