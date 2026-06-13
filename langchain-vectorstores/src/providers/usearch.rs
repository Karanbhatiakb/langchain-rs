//! USearch vector store implementation.
//!
//! USearch is a compact, efficient vector search library that supports
//! multiple distance metrics and SIMD-optimized search.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by USearch.
///
/// USearch is a small, fast vector search library built for similarity
/// search over dense vectors with metrics like cosine and Euclidean.
#[derive(Clone)]
pub struct USearchVectorStore {
    dimensions: usize,
    metric: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for USearchVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("USearchVectorStore")
            .field("dimensions", &self.dimensions)
            .field("metric", &self.metric)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl USearchVectorStore {
    /// Create a new `USearchVectorStore`.
    ///
    /// * `dimensions` — the dimensionality of the vectors.
    /// * `metric` — the distance metric (e.g. `"cos"`, `"l2"`, `"ip"`).
    /// * `embeddings` — the embedding model.
    pub fn new(
        dimensions: usize,
        metric: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            dimensions,
            metric: metric.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for USearchVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("USearchVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("USearchVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("USearchVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "USearchVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "USearchVectorStore is a stub; similarity_search_with_score returns empty"
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
            "USearchVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("USearchVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
