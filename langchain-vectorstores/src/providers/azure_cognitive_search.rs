//! Azure Cognitive Search vector store integration.
//!
//! Azure Cognitive Search (now part of Azure AI Search) provides vector
//! search capabilities alongside full-text search within managed search
//! indexes.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Azure Cognitive Search.
#[derive(Clone)]
pub struct AzureCognitiveSearchVectorStore {
    service_name: String,
    index_name: String,
    api_key: String,
    api_version: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for AzureCognitiveSearchVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureCognitiveSearchVectorStore")
            .field("service_name", &self.service_name)
            .field("index_name", &self.index_name)
            .field("api_key", &"***")
            .field("api_version", &self.api_version)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl AzureCognitiveSearchVectorStore {
    /// Create a new `AzureCognitiveSearchVectorStore`.
    ///
    /// * `service_name` — the Azure Search service name.
    /// * `index_name` — the search index name.
    /// * `api_key` — the admin / query API key.
    /// * `api_version` — the REST API version (e.g. `"2024-07-01"`).
    /// * `embeddings` — the embedding model.
    pub fn new(
        service_name: impl Into<String>,
        index_name: impl Into<String>,
        api_key: impl Into<String>,
        api_version: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            index_name: index_name.into(),
            api_key: api_key.into(),
            api_version: api_version.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for AzureCognitiveSearchVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!(
            "AzureCognitiveSearchVectorStore is a stub; add_texts returns empty"
        );
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!(
            "AzureCognitiveSearchVectorStore is a stub; add_documents returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!(
            "AzureCognitiveSearchVectorStore is a stub; similarity_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "AzureCognitiveSearchVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "AzureCognitiveSearchVectorStore is a stub; similarity_search_with_score returns empty"
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
            "AzureCognitiveSearchVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("AzureCognitiveSearchVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
