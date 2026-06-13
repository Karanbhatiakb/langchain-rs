//! IBM Cloud Object Storage vector store.
//!
//! Stub implementation — replace with real backend integration.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by IbmCosVector.
#[derive(Clone)]
pub struct IbmCosVectorStore {
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for IbmCosVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IbmCosVectorStore")
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl IbmCosVectorStore {
    /// Create a new `IbmCosVectorStore`.
    pub fn new(embeddings: Arc<dyn Embeddings>) -> Self {
        Self { embeddings }
    }
}

#[async_trait]
impl VectorStore for IbmCosVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("IbmCosVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("IbmCosVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("IbmCosVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!("IbmCosVectorStore is a stub; similarity_search_by_vector returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!("IbmCosVectorStore is a stub; similarity_search_with_score returns empty");
        Ok(Vec::new())
    }

    async fn max_marginal_relevance_search(
        &self,
        _query: &str,
        _k: usize,
        _fetch_k: usize,
        _lambda_mult: f32,
    ) -> Result<Vec<Document>> {
        tracing::warn!("IbmCosVectorStore is a stub; max_marginal_relevance_search returns empty");
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("IbmCosVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
