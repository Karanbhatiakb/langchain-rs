//! AnalyticDB for PostgreSQL vector store implementation.
//!
//! AnalyticDB for PostgreSQL is a distributed data warehousing service on
//! Alibaba Cloud that supports vector similarity search.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by AnalyticDB for PostgreSQL.
#[derive(Clone)]
pub struct AnalyticDbPostgresVectorStore {
    connection_string: String,
    table_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for AnalyticDbPostgresVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticDbPostgresVectorStore")
            .field("connection_string", &"***")
            .field("table_name", &self.table_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl AnalyticDbPostgresVectorStore {
    /// Create a new `AnalyticDbPostgresVectorStore`.
    ///
    /// * `connection_string` — the ADB PG connection string.
    /// * `table_name` — the table storing vectors.
    /// * `embeddings` — the embedding model.
    pub fn new(
        connection_string: impl Into<String>,
        table_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            connection_string: connection_string.into(),
            table_name: table_name.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for AnalyticDbPostgresVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("AnalyticDbPostgresVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("AnalyticDbPostgresVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("AnalyticDbPostgresVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "AnalyticDbPostgresVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "AnalyticDbPostgresVectorStore is a stub; similarity_search_with_score returns empty"
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
            "AnalyticDbPostgresVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("AnalyticDbPostgresVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
