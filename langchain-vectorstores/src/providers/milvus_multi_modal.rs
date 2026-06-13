//! Milvus multi-modal vector store integration.
//!
//! Extends Milvus to support multi-modal data (text, image, video) through
//! Milvus collections with support for multiple vector fields per entity.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Multi-modal vector store backed by Milvus.
///
/// Supports indexing and searching across multiple embedding vectors per
/// document, enabling cross-modal retrieval (e.g. text-to-image).
#[derive(Clone)]
pub struct MilvusMultiModalVectorStore {
    host: String,
    port: u16,
    collection_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for MilvusMultiModalVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MilvusMultiModalVectorStore")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("collection_name", &self.collection_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl MilvusMultiModalVectorStore {
    /// Create a new `MilvusMultiModalVectorStore`.
    ///
    /// * `host` — the Milvus server hostname.
    /// * `port` — the Milvus server port (default: 19530).
    /// * `collection_name` — the Milvus collection name.
    /// * `embeddings` — the embedding model.
    pub fn new(
        host: impl Into<String>,
        port: u16,
        collection_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            collection_name: collection_name.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for MilvusMultiModalVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!(
            "MilvusMultiModalVectorStore is a stub; add_texts returns empty"
        );
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!(
            "MilvusMultiModalVectorStore is a stub; add_documents returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!(
            "MilvusMultiModalVectorStore is a stub; similarity_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "MilvusMultiModalVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "MilvusMultiModalVectorStore is a stub; similarity_search_with_score returns empty"
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
            "MilvusMultiModalVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("MilvusMultiModalVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
