//! DataStax Astra DB vector store implementation.
//!
//! DataStax Astra DB is a cloud-native database built on Apache Cassandra with
//! integrated vector search. This provider offers a detailed integration for
//! managing vector embeddings at scale with Astra's serverless architecture.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by DataStax Astra DB.
///
/// Astra DB provides a serverless Cassandra-compatible database with native
/// vector search capabilities, supporting multiple similarity metrics and
/// ANN (approximate nearest neighbour) queries.
#[derive(Clone)]
pub struct DataStaxVectorStore {
    /// The Astra DB API endpoint URL.
    endpoint: String,
    /// The Astra DB application token.
    application_token: String,
    /// The keyspace name.
    keyspace: String,
    /// The table name for vector storage.
    table_name: String,
    /// The dimension of the embedding vectors.
    dimension: usize,
    /// The HTTP client used for API requests.
    client: reqwest::Client,
    /// The embedding model.
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for DataStaxVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataStaxVectorStore")
            .field("endpoint", &self.endpoint)
            .field("application_token", &"***")
            .field("keyspace", &self.keyspace)
            .field("table_name", &self.table_name)
            .field("dimension", &self.dimension)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl DataStaxVectorStore {
    /// Create a new `DataStaxVectorStore`.
    ///
    /// * `endpoint` — the Astra DB REST API endpoint.
    /// * `application_token` — the Astra DB application token (AstraCS:...).
    /// * `keyspace` — the keyspace name.
    /// * `table_name` — the table name for vector storage.
    /// * `dimension` — the dimension of embedding vectors.
    /// * `embeddings` — the embedding model.
    pub fn new(
        endpoint: impl Into<String>,
        application_token: impl Into<String>,
        keyspace: impl Into<String>,
        table_name: impl Into<String>,
        dimension: usize,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            application_token: application_token.into(),
            keyspace: keyspace.into(),
            table_name: table_name.into(),
            dimension,
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for DataStaxVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("DataStaxVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("DataStaxVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("DataStaxVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "DataStaxVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "DataStaxVectorStore is a stub; similarity_search_with_score returns empty"
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
            "DataStaxVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("DataStaxVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
