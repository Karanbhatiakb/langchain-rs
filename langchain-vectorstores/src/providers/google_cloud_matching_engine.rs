//! Google Vertex AI Matching Engine vector store implementation.
//!
//! Vertex AI Matching Engine provides scalable vector similarity search for
//! large-scale AI applications on Google Cloud.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Google Vertex AI Matching Engine.
#[derive(Clone)]
pub struct GoogleCloudMatchingEngineStore {
    project_id: String,
    location: String,
    index_id: String,
    endpoint_id: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for GoogleCloudMatchingEngineStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleCloudMatchingEngineStore")
            .field("project_id", &self.project_id)
            .field("location", &self.location)
            .field("index_id", &self.index_id)
            .field("endpoint_id", &self.endpoint_id)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl GoogleCloudMatchingEngineStore {
    /// Create a new `GoogleCloudMatchingEngineStore`.
    ///
    /// * `project_id` — the GCP project ID.
    /// * `location` — the GCP location.
    /// * `index_id` — the Matching Engine index ID.
    /// * `endpoint_id` — the Matching Engine endpoint ID.
    /// * `embeddings` — the embedding model.
    pub fn new(
        project_id: impl Into<String>,
        location: impl Into<String>,
        index_id: impl Into<String>,
        endpoint_id: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            location: location.into(),
            index_id: index_id.into(),
            endpoint_id: endpoint_id.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for GoogleCloudMatchingEngineStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("GoogleCloudMatchingEngineStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("GoogleCloudMatchingEngineStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("GoogleCloudMatchingEngineStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "GoogleCloudMatchingEngineStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "GoogleCloudMatchingEngineStore is a stub; similarity_search_with_score returns empty"
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
            "GoogleCloudMatchingEngineStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("GoogleCloudMatchingEngineStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
