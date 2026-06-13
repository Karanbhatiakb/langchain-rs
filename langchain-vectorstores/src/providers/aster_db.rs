//! Aster DB vector store implementation.
//!
//! Aster DB is a distributed vector database designed for AI workloads,
//! offering high-performance similarity search at scale.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Aster DB.
#[derive(Clone)]
pub struct AsterDBVectorStore {
    host: String,
    port: u16,
    database: String,
    collection: String,
    username: String,
    password: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for AsterDBVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsterDBVectorStore")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("collection", &self.collection)
            .field("username", &self.username)
            .field("password", &"***")
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl AsterDBVectorStore {
    /// Create a new `AsterDBVectorStore`.
    ///
    /// * `host` — the Aster DB host.
    /// * `port` — the Aster DB port.
    /// * `database` — the database name.
    /// * `collection` — the collection/table name.
    /// * `username` — the username for authentication.
    /// * `password` — the password for authentication.
    /// * `embeddings` — the embedding model.
    pub fn new(
        host: impl Into<String>,
        port: u16,
        database: impl Into<String>,
        collection: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            database: database.into(),
            collection: collection.into(),
            username: username.into(),
            password: password.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for AsterDBVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("AsterDBVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("AsterDBVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("AsterDBVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "AsterDBVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "AsterDBVectorStore is a stub; similarity_search_with_score returns empty"
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
            "AsterDBVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("AsterDBVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
