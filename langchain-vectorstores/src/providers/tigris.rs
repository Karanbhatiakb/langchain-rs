//! Tigris vector store implementation.
//!
//! Tigris is an open-source serverless NoSQL database and search platform
//! with built-in vector search capabilities.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Tigris.
#[derive(Clone)]
pub struct TigrisVectorStore {
    project_name: String,
    collection_name: String,
    token: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for TigrisVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TigrisVectorStore")
            .field("project_name", &self.project_name)
            .field("collection_name", &self.collection_name)
            .field("token", &"***")
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl TigrisVectorStore {
    /// Create a new `TigrisVectorStore`.
    ///
    /// * `project_name` — the Tigris project name.
    /// * `collection_name` — the collection name.
    /// * `token` — the Tigris API token.
    /// * `embeddings` — the embedding model.
    pub fn new(
        project_name: impl Into<String>,
        collection_name: impl Into<String>,
        token: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            project_name: project_name.into(),
            collection_name: collection_name.into(),
            token: token.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for TigrisVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("TigrisVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("TigrisVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("TigrisVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "TigrisVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "TigrisVectorStore is a stub; similarity_search_with_score returns empty"
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
            "TigrisVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("TigrisVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
