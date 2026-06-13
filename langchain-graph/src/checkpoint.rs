//! Checkpoint persistence for graph state.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use langchain_core::errors::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub state: Value,
    pub timestamp: DateTime<Utc>,
    pub parent_id: Option<Uuid>,
    pub metadata: Option<Value>,
}

impl Checkpoint {
    pub fn new(state: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            state,
            timestamp: Utc::now(),
            parent_id: None,
            metadata: None,
        }
    }

    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[async_trait]
pub trait Checkpointer: Send + Sync {
    async fn save(&self, checkpoint_id: &str, state: &Value) -> Result<()>;
    async fn load(&self, checkpoint_id: &str) -> Result<Option<Value>>;
    async fn list(&self) -> Result<Vec<String>>;
    async fn delete(&self, checkpoint_id: &str) -> Result<()>;
}

pub struct MemoryCheckpointer {
    store: DashMap<String, Value>,
}

impl MemoryCheckpointer {
    pub fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }
}

impl Default for MemoryCheckpointer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Checkpointer for MemoryCheckpointer {
    async fn save(&self, checkpoint_id: &str, state: &Value) -> Result<()> {
        self.store.insert(checkpoint_id.to_string(), state.clone());
        Ok(())
    }

    async fn load(&self, checkpoint_id: &str) -> Result<Option<Value>> {
        Ok(self.store.get(checkpoint_id).map(|v| v.value().clone()))
    }

    async fn list(&self) -> Result<Vec<String>> {
        Ok(self.store.iter().map(|e| e.key().clone()).collect())
    }

    async fn delete(&self, checkpoint_id: &str) -> Result<()> {
        self.store.remove(checkpoint_id);
        Ok(())
    }
}
