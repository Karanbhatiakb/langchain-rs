//! Elastic Vector Search vector store implementation.
//!
//! Elastic Vector Search provides native dense vector and sparse vector
//! similarity search capabilities within Elasticsearch.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Elastic Vector Search.
#[derive(Clone)]
pub struct ElasticVectorSearchStore {
    cloud_id: String,
    api_key: String,
    index_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for ElasticVectorSearchStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElasticVectorSearchStore")
            .field("cloud_id", &self.cloud_id)
            .field("api_key", &"***")
            .field("index_name", &self.index_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl ElasticVectorSearchStore {
    /// Create a new `ElasticVectorSearchStore`.
    ///
    /// * `cloud_id` — the Elastic Cloud ID.
    /// * `api_key` — the API key.
    /// * `index_name` — the index name.
    /// * `embeddings` — the embedding model.
    pub fn new(
        cloud_id: impl Into<String>,
        api_key: impl Into<String>,
        index_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            cloud_id: cloud_id.into(),
            api_key: api_key.into(),
            index_name: index_name.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for ElasticVectorSearchStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("ElasticVectorSearchStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("ElasticVectorSearchStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("ElasticVectorSearchStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "ElasticVectorSearchStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "ElasticVectorSearchStore is a stub; similarity_search_with_score returns empty"
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
            "ElasticVectorSearchStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("ElasticVectorSearchStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
