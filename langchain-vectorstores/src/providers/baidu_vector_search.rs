//! Baidu Vector Search vector store implementation.
//!
//! Baidu Vector Search is a fully managed vector database service on Baidu AI
//! Cloud that enables high-performance similarity search.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Baidu Vector Search.
#[derive(Clone)]
pub struct BaiduVectorSearchStore {
    endpoint: String,
    api_key: String,
    database_name: String,
    collection_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for BaiduVectorSearchStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaiduVectorSearchStore")
            .field("endpoint", &self.endpoint)
            .field("api_key", &"***")
            .field("database_name", &self.database_name)
            .field("collection_name", &self.collection_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl BaiduVectorSearchStore {
    /// Create a new `BaiduVectorSearchStore`.
    ///
    /// * `endpoint` — the Baidu Vector Search endpoint.
    /// * `api_key` — the API key.
    /// * `database_name` — the database name.
    /// * `collection_name` — the collection name.
    /// * `embeddings` — the embedding model.
    pub fn new(
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
        database_name: impl Into<String>,
        collection_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            database_name: database_name.into(),
            collection_name: collection_name.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for BaiduVectorSearchStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("BaiduVectorSearchStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("BaiduVectorSearchStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("BaiduVectorSearchStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "BaiduVectorSearchStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "BaiduVectorSearchStore is a stub; similarity_search_with_score returns empty"
        );
        Ok(Vec::new())
    }

    async fn max_marginal_relevance_search(
        &self,
        _query: &str,
        _k: usize,
        _fetch_k: usize,
        _lambda_mult: f32,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "BaiduVectorSearchStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("BaiduVectorSearchStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
