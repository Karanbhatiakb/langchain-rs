//! Tencent VectorDB vector store implementation.
//!
//! Tencent VectorDB is a fully managed vector database service on Tencent
//! Cloud, offering high-performance similarity search and hybrid queries.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Tencent VectorDB.
#[derive(Clone)]
pub struct TencentVectorDBVectorStore {
    url: String,
    api_key: String,
    collection_name: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for TencentVectorDBVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TencentVectorDBVectorStore")
            .field("url", &self.url)
            .field("api_key", &"***")
            .field("collection_name", &self.collection_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl TencentVectorDBVectorStore {
    /// Create a new `TencentVectorDBVectorStore`.
    ///
    /// * `url` — the Tencent VectorDB service URL.
    /// * `api_key` — the API key.
    /// * `collection_name` — the collection name.
    /// * `embeddings` — the embedding model.
    pub fn new(
        url: impl Into<String>,
        api_key: impl Into<String>,
        collection_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            api_key: api_key.into(),
            collection_name: collection_name.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for TencentVectorDBVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!(
            "TencentVectorDBVectorStore is a stub; add_texts returns empty"
        );
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!(
            "TencentVectorDBVectorStore is a stub; add_documents returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!(
            "TencentVectorDBVectorStore is a stub; similarity_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "TencentVectorDBVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "TencentVectorDBVectorStore is a stub; similarity_search_with_score returns empty"
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
            "TencentVectorDBVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("TencentVectorDBVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
