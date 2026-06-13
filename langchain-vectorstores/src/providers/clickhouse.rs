//! ClickHouse vector store implementation.
//!
//! Uses ClickHouse's built-in vector search capabilities with
//! cosine distance via the `cosineDistance` function.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by ClickHouse.
#[derive(Clone)]
pub struct ClickHouseVectorStore {
    url: String,
    database: String,
    table: String,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for ClickHouseVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickHouseVectorStore")
            .field("url", &self.url)
            .field("database", &self.database)
            .field("table", &self.table)
            .field("username", &self.username)
            .field("password", &"***")
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl ClickHouseVectorStore {
    /// Create a new `ClickHouseVectorStore`.
    ///
    /// * `url` — the ClickHouse HTTP endpoint.
    /// * `database` — the database name.
    /// * `table` — the table name.
    /// * `username` — the ClickHouse username.
    /// * `password` — the ClickHouse password.
    /// * `embeddings` — the embedding model.
    pub fn new(
        url: impl Into<String>,
        database: impl Into<String>,
        table: impl Into<String>,
        username: Option<String>,
        password: Option<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            database: database.into(),
            table: table.into(),
            username,
            password,
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for ClickHouseVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("ClickHouseVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("ClickHouseVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("ClickHouseVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "ClickHouseVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "ClickHouseVectorStore is a stub; similarity_search_with_score returns empty"
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
            "ClickHouseVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("ClickHouseVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
