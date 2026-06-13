//! Clarifai vector store integration.
//!
//! Clarifai provides an AI platform with a vector database for searching
//! and retrieving across text, image, video, and audio inputs.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Clarifai.
#[derive(Clone)]
pub struct ClarifaiVectorStore {
    api_key: String,
    user_id: String,
    app_id: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for ClarifaiVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClarifaiVectorStore")
            .field("api_key", &"***")
            .field("user_id", &self.user_id)
            .field("app_id", &self.app_id)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl ClarifaiVectorStore {
    /// Create a new `ClarifaiVectorStore`.
    ///
    /// * `api_key` — a Clarifai PAT (Personal Access Token).
    /// * `user_id` — the Clarifai user ID.
    /// * `app_id` — the Clarifai application ID.
    /// * `embeddings` — the embedding model.
    pub fn new(
        api_key: impl Into<String>,
        user_id: impl Into<String>,
        app_id: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            user_id: user_id.into(),
            app_id: app_id.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for ClarifaiVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("ClarifaiVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("ClarifaiVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("ClarifaiVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "ClarifaiVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "ClarifaiVectorStore is a stub; similarity_search_with_score returns empty"
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
            "ClarifaiVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("ClarifaiVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
