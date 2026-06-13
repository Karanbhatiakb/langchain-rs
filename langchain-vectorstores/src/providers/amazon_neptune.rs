//! Amazon Neptune graph vector store implementation.
//!
//! Amazon Neptune is a fully managed graph database service. This provider
//! integrates Neptune's graph capabilities with vector similarity search.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Amazon Neptune.
#[derive(Clone)]
pub struct AmazonNeptuneVectorStore {
    endpoint: String,
    port: u16,
    graph_name: String,
    region: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for AmazonNeptuneVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AmazonNeptuneVectorStore")
            .field("endpoint", &self.endpoint)
            .field("port", &self.port)
            .field("graph_name", &self.graph_name)
            .field("region", &self.region)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl AmazonNeptuneVectorStore {
    /// Create a new `AmazonNeptuneVectorStore`.
    ///
    /// * `endpoint` — the Neptune cluster endpoint.
    /// * `port` — the Neptune cluster port (default 8182).
    /// * `graph_name` — the graph name.
    /// * `region` — the AWS region.
    /// * `embeddings` — the embedding model.
    pub fn new(
        endpoint: impl Into<String>,
        port: u16,
        graph_name: impl Into<String>,
        region: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            port,
            graph_name: graph_name.into(),
            region: region.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for AmazonNeptuneVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("AmazonNeptuneVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("AmazonNeptuneVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("AmazonNeptuneVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "AmazonNeptuneVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "AmazonNeptuneVectorStore is a stub; similarity_search_with_score returns empty"
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
            "AmazonNeptuneVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("AmazonNeptuneVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
