//! Redis vector store implementation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

#[allow(dead_code)]
pub struct RedisVectorStore {
    url: String,
    index_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl RedisVectorStore {
    pub fn new(
        url: impl Into<String>,
        index_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            url: url.into(),
            index_name: index_name.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for RedisVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        Err(ChainError::VectorStoreError(
            "RedisVectorStore requires the `redis` feature with a redis::Client connection - use RedisVectorStore::from_client()".into(),
        ))
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        Err(ChainError::VectorStoreError(
            "RedisVectorStore requires the `redis` feature with a redis::Client connection".into(),
        ))
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        Err(ChainError::VectorStoreError(
            "RedisVectorStore requires the `redis` feature with a redis::Client connection".into(),
        ))
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        Err(ChainError::VectorStoreError(
            "RedisVectorStore requires the `redis` feature with a redis::Client connection".into(),
        ))
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        Err(ChainError::VectorStoreError(
            "RedisVectorStore requires the `redis` feature with a redis::Client connection".into(),
        ))
    }

    async fn max_marginal_relevance_search(
        &self,
        _query: &str,
        _k: usize,
        _fetch_k: usize,
        _lambda_mult: f32,
    ) -> Result<Vec<Document>> {
        Err(ChainError::VectorStoreError(
            "RedisVectorStore requires the `redis` feature with a redis::Client connection".into(),
        ))
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        Err(ChainError::VectorStoreError(
            "RedisVectorStore requires the `redis` feature with a redis::Client connection".into(),
        ))
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
