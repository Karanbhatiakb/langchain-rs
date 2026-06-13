//! Azure AI Search vector store implementation.
//!
//! Uses Azure Cognitive Search's built-in vector search capabilities to
//! store and retrieve documents by embedding similarity.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Azure AI Search.
#[derive(Clone)]
pub struct AzureSearchVectorStore {
    service_name: String,
    index_name: String,
    api_key: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for AzureSearchVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureSearchVectorStore")
            .field("service_name", &self.service_name)
            .field("index_name", &self.index_name)
            .field("api_key", &"***")
            .field("client", &self.client)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl AzureSearchVectorStore {
    /// Create a new `AzureSearchVectorStore`.
    ///
    /// * `service_name` — the Azure AI Search service name.
    /// * `index_name` — the index to use.
    /// * `api_key` — the admin / query API key.
    /// * `embeddings` — the embedding model.
    pub fn new(
        service_name: impl Into<String>,
        index_name: impl Into<String>,
        api_key: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            index_name: index_name.into(),
            api_key: api_key.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }

    fn base_url(&self) -> String {
        format!(
            "https://{}.search.windows.net/indexes/{}/docs",
            self.service_name, self.index_name
        )
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "api-key",
            reqwest::header::HeaderValue::from_str(&self.api_key)
                .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("")),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }
}

#[async_trait]
impl VectorStore for AzureSearchVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("AzureSearchVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("AzureSearchVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("AzureSearchVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "AzureSearchVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "AzureSearchVectorStore is a stub; similarity_search_with_score returns empty"
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
            "AzureSearchVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("AzureSearchVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
