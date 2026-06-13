//! Document stores for parent-document and multi-vector retrievers.

use crate::documents::Document;
use crate::errors::*;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

#[async_trait]
pub trait DocStore: Send + Sync {
    async fn search(&self, search: &str) -> Result<Vec<Document>>;
    async fn get(&self, id: &str) -> Result<Option<Document>>;
    async fn delete(&self, ids: Vec<&str>) -> Result<()>;
}

#[async_trait]
pub trait AddableDocStore: DocStore {
    async fn add(&self, docs: Vec<Document>) -> Result<Vec<String>>;
}

pub struct InMemoryDocStore {
    store: Arc<DashMap<String, Document>>,
}

impl InMemoryDocStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryDocStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocStore for InMemoryDocStore {
    async fn search(&self, search: &str) -> Result<Vec<Document>> {
        let search_lower = search.to_lowercase();
        Ok(self
            .store
            .iter()
            .filter(|entry| {
                entry
                    .value()
                    .page_content
                    .to_lowercase()
                    .contains(&search_lower)
            })
            .map(|entry| entry.value().clone())
            .collect())
    }

    async fn get(&self, id: &str) -> Result<Option<Document>> {
        Ok(self.store.get(id).map(|v| v.value().clone()))
    }

    async fn delete(&self, ids: Vec<&str>) -> Result<()> {
        for id in ids {
            self.store.remove(id);
        }
        Ok(())
    }
}

#[async_trait]
impl AddableDocStore for InMemoryDocStore {
    async fn add(&self, docs: Vec<Document>) -> Result<Vec<String>> {
        let mut ids = Vec::with_capacity(docs.len());
        for doc in docs {
            let id = uuid::Uuid::new_v4().to_string();
            self.store.insert(id.clone(), doc);
            ids.push(id);
        }
        Ok(ids)
    }
}
