//! Chroma Cloud vector store implementation.
//!
//! Chroma Cloud is the hosted version of the Chroma vector database.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Chroma Cloud.
#[derive(Clone)]
pub struct ChromaCloudVectorStore {
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for ChromaCloudVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChromaCloudVectorStore")
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl ChromaCloudVectorStore {
    /// Create a new `ChromaCloudVectorStore`.
    pub fn new(embeddings: Arc<dyn Embeddings>) -> Self {
        Self { embeddings }
    }
}

#[async_trait]
impl VectorStore for ChromaCloudVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("ChromaCloudVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("ChromaCloudVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("ChromaCloudVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!("ChromaCloudVectorStore is a stub; similarity_search_by_vector returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!("ChromaCloudVectorStore is a stub; similarity_search_with_score returns empty");
        Ok(Vec::new())
    }

    async fn max_marginal_relevance_search(
        &self,
        _query: &str,
        _k: usize,
        _fetch_k: usize,
        _lambda_mult: f32,
    ) -> Result<Vec<Document>> {
        tracing::warn!("ChromaCloudVectorStore is a stub; max_marginal_relevance_search returns empty");
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("ChromaCloudVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
