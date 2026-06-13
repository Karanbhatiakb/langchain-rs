//! Alibaba Cloud Search vector store implementation.
//!
//! Alibaba Cloud Search (also known as Tair Vector) provides vector similarity
//! search capabilities within the Alibaba Cloud ecosystem.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Alibaba Cloud Search / Tair Vector.
#[derive(Clone)]
pub struct AlibabaCloudSearchVectorStore {
    endpoint: String,
    instance_id: String,
    username: String,
    password: String,
    index_name: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for AlibabaCloudSearchVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlibabaCloudSearchVectorStore")
            .field("endpoint", &self.endpoint)
            .field("instance_id", &self.instance_id)
            .field("username", &self.username)
            .field("password", &"***")
            .field("index_name", &self.index_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl AlibabaCloudSearchVectorStore {
    /// Create a new `AlibabaCloudSearchVectorStore`.
    ///
    /// * `endpoint` — the service endpoint.
    /// * `instance_id` — the Tair instance ID.
    /// * `username` — the username for authentication.
    /// * `password` — the password for authentication.
    /// * `index_name` — the vector index name.
    /// * `embeddings` — the embedding model.
    pub fn new(
        endpoint: impl Into<String>,
        instance_id: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        index_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            instance_id: instance_id.into(),
            username: username.into(),
            password: password.into(),
            index_name: index_name.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for AlibabaCloudSearchVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("AlibabaCloudSearchVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("AlibabaCloudSearchVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!(
            "AlibabaCloudSearchVectorStore is a stub; similarity_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "AlibabaCloudSearchVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "AlibabaCloudSearchVectorStore is a stub; similarity_search_with_score returns empty"
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
            "AlibabaCloudSearchVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("AlibabaCloudSearchVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
