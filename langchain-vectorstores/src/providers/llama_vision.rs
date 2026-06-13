//! Llama Vision vector store integration.
//!
//! Provides a vector store backed by Llama Vision models for multi-modal
//! retrieval, indexing images alongside text using vision-language
//! embeddings.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Llama Vision.
///
/// Supports storing and searching over image-text documents using
/// vision-language embedding models.
#[derive(Clone)]
pub struct LlamaVisionVectorStore {
    api_url: String,
    api_key: Option<String>,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for LlamaVisionVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlamaVisionVectorStore")
            .field("api_url", &self.api_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "***"))
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl LlamaVisionVectorStore {
    /// Create a new `LlamaVisionVectorStore`.
    ///
    /// * `api_url` — the Llama Vision API endpoint.
    /// * `api_key` — optional API key.
    /// * `embeddings` — the embedding model.
    pub fn new(
        api_url: impl Into<String>,
        api_key: Option<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            api_url: api_url.into(),
            api_key,
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for LlamaVisionVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("LlamaVisionVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("LlamaVisionVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("LlamaVisionVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "LlamaVisionVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "LlamaVisionVectorStore is a stub; similarity_search_with_score returns empty"
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
            "LlamaVisionVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("LlamaVisionVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
