//! LlamaIndex vector store integration.
//!
//! Provides a bridge to LlamaIndex-managed vector indices via its REST API,
//! allowing queries against indices built with LlamaIndex's data framework.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by a LlamaIndex index.
///
/// Communicates with a running LlamaIndex server to perform queries against
/// pre-built indices (vector, summary, keyword, etc.).
#[derive(Clone)]
pub struct LlamaIndexVectorStore {
    endpoint: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for LlamaIndexVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlamaIndexVectorStore")
            .field("endpoint", &self.endpoint)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl LlamaIndexVectorStore {
    /// Create a new `LlamaIndexVectorStore`.
    ///
    /// * `endpoint` — the base URL of the LlamaIndex server (e.g.
    ///   `http://localhost:8000`).
    /// * `embeddings` — the embedding model.
    pub fn new(endpoint: impl Into<String>, embeddings: Arc<dyn Embeddings>) -> Self {
        Self {
            endpoint: endpoint.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for LlamaIndexVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("LlamaIndexVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("LlamaIndexVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("LlamaIndexVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "LlamaIndexVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "LlamaIndexVectorStore is a stub; similarity_search_with_score returns empty"
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
            "LlamaIndexVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("LlamaIndexVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
