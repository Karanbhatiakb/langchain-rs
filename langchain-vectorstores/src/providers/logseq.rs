//! Logseq knowledge base vector store integration.
//!
//! Provides a vector store that indexes Logseq markdown pages and blocks,
//! enabling semantic search over personal knowledge graphs built with
//! Logseq.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by a Logseq knowledge base.
#[derive(Clone)]
pub struct LogseqVectorStore {
    graph_path: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for LogseqVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogseqVectorStore")
            .field("graph_path", &self.graph_path)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl LogseqVectorStore {
    /// Create a new `LogseqVectorStore`.
    ///
    /// * `graph_path` — the filesystem path to the Logseq graph directory.
    /// * `embeddings` — the embedding model.
    pub fn new(graph_path: impl Into<String>, embeddings: Arc<dyn Embeddings>) -> Self {
        Self {
            graph_path: graph_path.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for LogseqVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("LogseqVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("LogseqVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("LogseqVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "LogseqVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "LogseqVectorStore is a stub; similarity_search_with_score returns empty"
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
            "LogseqVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("LogseqVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
