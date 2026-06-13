//! Activeloop Deep Lake vector store integration.
//!
//! Deep Lake is a vector database for AI that stores embeddings, metadata,
//! and data in a unified format.  This module targets the Activeloop REST
//! API (`api.activeloop.ai`).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Activeloop Deep Lake.
#[derive(Clone)]
pub struct ActiveloopVectorStore {
    dataset_path: String,
    api_key: Option<String>,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for ActiveloopVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveloopVectorStore")
            .field("dataset_path", &self.dataset_path)
            .field("api_key", &self.api_key.as_ref().map(|_| "***"))
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl ActiveloopVectorStore {
    /// Create a new `ActiveloopVectorStore`.
    ///
    /// * `dataset_path` — the Deep Lake dataset path (e.g.
    ///   `hub://user/dataset`).
    /// * `api_key` — optional Activeloop API key.
    /// * `embeddings` — the embedding model.
    pub fn new(
        dataset_path: impl Into<String>,
        api_key: Option<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            dataset_path: dataset_path.into(),
            api_key,
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for ActiveloopVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("ActiveloopVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("ActiveloopVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("ActiveloopVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "ActiveloopVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "ActiveloopVectorStore is a stub; similarity_search_with_score returns empty"
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
            "ActiveloopVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("ActiveloopVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
