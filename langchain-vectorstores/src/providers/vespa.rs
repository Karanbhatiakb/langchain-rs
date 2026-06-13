//! Vespa vector store implementation.
//!
//! Vespa is a fully featured search engine and vector database that supports
//! approximate nearest neighbour search with real-time writes.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Vespa.
#[derive(Clone)]
pub struct VespaVectorStore {
    url: String,
    index_name: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for VespaVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VespaVectorStore")
            .field("url", &self.url)
            .field("index_name", &self.index_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl VespaVectorStore {
    /// Create a new `VespaVectorStore`.
    ///
    /// * `url` — the Vespa endpoint URL.
    /// * `index_name` — the document type / schema name.
    /// * `embeddings` — the embedding model.
    pub fn new(
        url: impl Into<String>,
        index_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            index_name: index_name.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for VespaVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("VespaVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("VespaVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("VespaVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!("VespaVectorStore is a stub; similarity_search_by_vector returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!("VespaVectorStore is a stub; similarity_search_with_score returns empty");
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
            "VespaVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("VespaVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
