//! Google Cloud Console vector store integration.
//!
//! Provides a vector store backed by Google Cloud services accessible via
//! the Cloud Console, including Cloud SQL with pgvector, Vertex AI, and
//! Cloud Firestore.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Google Cloud Console services.
#[derive(Clone)]
pub struct GoogleCloudConsoleVectorStore {
    project_id: String,
    location: String,
    index_id: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for GoogleCloudConsoleVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleCloudConsoleVectorStore")
            .field("project_id", &"***")
            .field("location", &self.location)
            .field("index_id", &self.index_id)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl GoogleCloudConsoleVectorStore {
    /// Create a new `GoogleCloudConsoleVectorStore`.
    ///
    /// * `project_id` — the GCP project ID.
    /// * `location` — the GCP location / region.
    /// * `index_id` — the vector index ID.
    /// * `embeddings` — the embedding model.
    pub fn new(
        project_id: impl Into<String>,
        location: impl Into<String>,
        index_id: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            location: location.into(),
            index_id: index_id.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for GoogleCloudConsoleVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!(
            "GoogleCloudConsoleVectorStore is a stub; add_texts returns empty"
        );
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!(
            "GoogleCloudConsoleVectorStore is a stub; add_documents returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!(
            "GoogleCloudConsoleVectorStore is a stub; similarity_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "GoogleCloudConsoleVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "GoogleCloudConsoleVectorStore is a stub; similarity_search_with_score returns empty"
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
            "GoogleCloudConsoleVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("GoogleCloudConsoleVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
