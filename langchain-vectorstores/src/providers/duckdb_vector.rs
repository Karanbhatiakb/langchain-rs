//! DuckDB vector store implementation.
//!
//! DuckDB is an in-process SQL OLAP database. This provider adds vector
//! similarity search using DuckDB's embedding support for local analytics.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by DuckDB.
#[derive(Clone)]
pub struct DuckDBVectorStore {
    path: String,
    table_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for DuckDBVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DuckDBVectorStore")
            .field("path", &self.path)
            .field("table_name", &self.table_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl DuckDBVectorStore {
    /// Create a new `DuckDBVectorStore`.
    ///
    /// * `path` — the DuckDB database file path.
    /// * `table_name` — the table name for vector storage.
    /// * `embeddings` — the embedding model.
    pub fn new(
        path: impl Into<String>,
        table_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            path: path.into(),
            table_name: table_name.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for DuckDBVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("DuckDBVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("DuckDBVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("DuckDBVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "DuckDBVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "DuckDBVectorStore is a stub; similarity_search_with_score returns empty"
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
            "DuckDBVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("DuckDBVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
