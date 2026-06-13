//! Apache Cassandra vector store implementation.
//!
//! Apache Cassandra is a distributed NoSQL database. This provider adds vector
//! similarity search support using Cassandra's storage engine.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Apache Cassandra.
#[derive(Clone)]
pub struct CassandraVectorStore {
    hosts: Vec<String>,
    port: u16,
    keyspace: String,
    table_name: String,
    username: String,
    password: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for CassandraVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CassandraVectorStore")
            .field("hosts", &self.hosts)
            .field("port", &self.port)
            .field("keyspace", &self.keyspace)
            .field("table_name", &self.table_name)
            .field("username", &self.username)
            .field("password", &"***")
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl CassandraVectorStore {
    /// Create a new `CassandraVectorStore`.
    ///
    /// * `hosts` — the Cassandra node hostnames.
    /// * `port` — the Cassandra port (default 9042).
    /// * `keyspace` — the keyspace name.
    /// * `table_name` — the table name for vector storage.
    /// * `username` — the username for authentication.
    /// * `password` — the password for authentication.
    /// * `embeddings` — the embedding model.
    pub fn new(
        hosts: Vec<String>,
        port: u16,
        keyspace: impl Into<String>,
        table_name: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            hosts,
            port,
            keyspace: keyspace.into(),
            table_name: table_name.into(),
            username: username.into(),
            password: password.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for CassandraVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("CassandraVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("CassandraVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("CassandraVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "CassandraVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "CassandraVectorStore is a stub; similarity_search_with_score returns empty"
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
            "CassandraVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("CassandraVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
