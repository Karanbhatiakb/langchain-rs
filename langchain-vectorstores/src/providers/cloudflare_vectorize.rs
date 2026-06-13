//! Cloudflare Vectorize vector store integration.
//!
//! Cloudflare Vectorize is a globally distributed vector database built on
//! Cloudflare's edge network, accessible via the Cloudflare API.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Cloudflare Vectorize.
#[derive(Clone)]
pub struct CloudflareVectorizeVectorStore {
    account_id: String,
    api_token: String,
    index_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for CloudflareVectorizeVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloudflareVectorizeVectorStore")
            .field("account_id", &self.account_id)
            .field("api_token", &"***")
            .field("index_name", &self.index_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl CloudflareVectorizeVectorStore {
    /// Create a new `CloudflareVectorizeVectorStore`.
    ///
    /// * `account_id` — the Cloudflare account ID.
    /// * `api_token` — a Cloudflare API token with Vectorize permissions.
    /// * `index_name` — the Vectorize index name.
    /// * `embeddings` — the embedding model.
    pub fn new(
        account_id: impl Into<String>,
        api_token: impl Into<String>,
        index_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            api_token: api_token.into(),
            index_name: index_name.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for CloudflareVectorizeVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!(
            "CloudflareVectorizeVectorStore is a stub; add_texts returns empty"
        );
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!(
            "CloudflareVectorizeVectorStore is a stub; add_documents returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!(
            "CloudflareVectorizeVectorStore is a stub; similarity_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "CloudflareVectorizeVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "CloudflareVectorizeVectorStore is a stub; similarity_search_with_score returns empty"
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
            "CloudflareVectorizeVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("CloudflareVectorizeVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
