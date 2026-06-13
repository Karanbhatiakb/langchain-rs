//! Google Vertex AI Vector Search (VEARC) integration.
//!
//! Vertex AI Vector Search provides a managed vector database on Google
//! Cloud for low-latency similarity search at scale, using a dedicated
//! index and deployed index endpoint.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Google Vertex AI Vector Search.
#[derive(Clone)]
pub struct VertexAIVectorSearchVectorStore {
    project_id: String,
    location: String,
    index_endpoint: String,
    deployed_index_id: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for VertexAIVectorSearchVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexAIVectorSearchVectorStore")
            .field("project_id", &"***")
            .field("location", &self.location)
            .field("index_endpoint", &self.index_endpoint)
            .field("deployed_index_id", &self.deployed_index_id)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl VertexAIVectorSearchVectorStore {
    /// Create a new `VertexAIVectorSearchVectorStore`.
    ///
    /// * `project_id` — the GCP project ID.
    /// * `location` — the GCP region (e.g. `us-central1`).
    /// * `index_endpoint` — the deployed index endpoint name.
    /// * `deployed_index_id` — the deployed index ID.
    /// * `embeddings` — the embedding model.
    pub fn new(
        project_id: impl Into<String>,
        location: impl Into<String>,
        index_endpoint: impl Into<String>,
        deployed_index_id: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            location: location.into(),
            index_endpoint: index_endpoint.into(),
            deployed_index_id: deployed_index_id.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for VertexAIVectorSearchVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!(
            "VertexAIVectorSearchVectorStore is a stub; add_texts returns empty"
        );
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!(
            "VertexAIVectorSearchVectorStore is a stub; add_documents returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!(
            "VertexAIVectorSearchVectorStore is a stub; similarity_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "VertexAIVectorSearchVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "VertexAIVectorSearchVectorStore is a stub; similarity_search_with_score returns empty"
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
            "VertexAIVectorSearchVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!(
            "VertexAIVectorSearchVectorStore is a stub; delete does nothing"
        );
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
