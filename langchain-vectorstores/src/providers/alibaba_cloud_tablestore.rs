//! Alibaba Cloud TableStore vector store implementation.
//!
//! Alibaba Cloud TableStore (Tablestore) is a fully managed NoSQL database that
//! supports vector similarity search for AI-powered applications.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Alibaba Cloud TableStore.
#[derive(Clone)]
pub struct AlibabaCloudTableStoreVectorStore {
    endpoint: String,
    instance_name: String,
    access_key_id: String,
    access_key_secret: String,
    table_name: String,
    client: reqwest::Client,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for AlibabaCloudTableStoreVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlibabaCloudTableStoreVectorStore")
            .field("endpoint", &self.endpoint)
            .field("instance_name", &self.instance_name)
            .field("access_key_id", &self.access_key_id)
            .field("access_key_secret", &"***")
            .field("table_name", &self.table_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl AlibabaCloudTableStoreVectorStore {
    /// Create a new `AlibabaCloudTableStoreVectorStore`.
    ///
    /// * `endpoint` — the TableStore endpoint.
    /// * `instance_name` — the TableStore instance name.
    /// * `access_key_id` — the Alibaba Cloud access key ID.
    /// * `access_key_secret` — the Alibaba Cloud access key secret.
    /// * `table_name` — the table name for vector storage.
    /// * `embeddings` — the embedding model.
    pub fn new(
        endpoint: impl Into<String>,
        instance_name: impl Into<String>,
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        table_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            instance_name: instance_name.into(),
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            table_name: table_name.into(),
            client: reqwest::Client::new(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for AlibabaCloudTableStoreVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!(
            "AlibabaCloudTableStoreVectorStore is a stub; add_texts returns empty"
        );
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!(
            "AlibabaCloudTableStoreVectorStore is a stub; add_documents returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!(
            "AlibabaCloudTableStoreVectorStore is a stub; similarity_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "AlibabaCloudTableStoreVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "AlibabaCloudTableStoreVectorStore is a stub; similarity_search_with_score returns empty"
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
            "AlibabaCloudTableStoreVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("AlibabaCloudTableStoreVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
