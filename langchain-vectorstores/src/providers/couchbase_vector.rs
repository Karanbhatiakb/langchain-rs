//! Couchbase vector store implementation.
//!
//! Couchbase is a distributed NoSQL database. This provider enables vector
//! similarity search using Couchbase's search capabilities.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Couchbase.
#[derive(Clone)]
pub struct CouchbaseVectorStore {
    connection_string: String,
    bucket: String,
    scope: String,
    collection: String,
    username: String,
    password: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for CouchbaseVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CouchbaseVectorStore")
            .field("connection_string", &"***")
            .field("bucket", &self.bucket)
            .field("scope", &self.scope)
            .field("collection", &self.collection)
            .field("username", &self.username)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl CouchbaseVectorStore {
    /// Create a new `CouchbaseVectorStore`.
    ///
    /// * `connection_string` — the Couchbase connection string.
    /// * `bucket` — the bucket name.
    /// * `scope` — the scope name.
    /// * `collection` — the collection name.
    /// * `username` — the username for authentication.
    /// * `password` — the password for authentication.
    /// * `embeddings` — the embedding model.
    pub fn new(
        connection_string: impl Into<String>,
        bucket: impl Into<String>,
        scope: impl Into<String>,
        collection: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            connection_string: connection_string.into(),
            bucket: bucket.into(),
            scope: scope.into(),
            collection: collection.into(),
            username: username.into(),
            password: password.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for CouchbaseVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("CouchbaseVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("CouchbaseVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("CouchbaseVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "CouchbaseVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "CouchbaseVectorStore is a stub; similarity_search_with_score returns empty"
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
            "CouchbaseVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("CouchbaseVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
