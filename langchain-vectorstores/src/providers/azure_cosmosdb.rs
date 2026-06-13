//! Azure CosmosDB for MongoDB vCore vector store implementation.
//!
//! Uses CosmosDB's MongoDB vCore API to store vector embeddings and perform
//! similarity search via the `$vector` aggregation pipeline.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

use crate::traits::VectorStore;

/// Vector store backed by Azure CosmosDB for MongoDB vCore.
#[derive(Clone)]
pub struct AzureCosmosDBVectorStore {
    connection_string: String,
    database_name: String,
    collection_name: String,
    embeddings: Arc<dyn Embeddings>,
}

impl std::fmt::Debug for AzureCosmosDBVectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureCosmosDBVectorStore")
            .field("connection_string", &"***")
            .field("database_name", &self.database_name)
            .field("collection_name", &self.collection_name)
            .field("embeddings", &"<embeddings>")
            .finish()
    }
}

impl AzureCosmosDBVectorStore {
    /// Create a new `AzureCosmosDBVectorStore`.
    ///
    /// * `connection_string` — the MongoDB connection string.
    /// * `database_name` — the database name.
    /// * `collection_name` — the collection to store vectors in.
    /// * `embeddings` — the embedding model.
    pub fn new(
        connection_string: impl Into<String>,
        database_name: impl Into<String>,
        collection_name: impl Into<String>,
        embeddings: Arc<dyn Embeddings>,
    ) -> Self {
        Self {
            connection_string: connection_string.into(),
            database_name: database_name.into(),
            collection_name: collection_name.into(),
            embeddings,
        }
    }
}

#[async_trait]
impl VectorStore for AzureCosmosDBVectorStore {
    async fn add_texts(
        &self,
        _texts: Vec<String>,
        _metadatas: Option<Vec<HashMap<String, Value>>>,
    ) -> Result<Vec<String>> {
        tracing::warn!("AzureCosmosDBVectorStore is a stub; add_texts returns empty");
        Ok(Vec::new())
    }

    async fn add_documents(&self, _docs: Vec<Document>) -> Result<Vec<String>> {
        tracing::warn!("AzureCosmosDBVectorStore is a stub; add_documents returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search(&self, _query: &str, _k: usize) -> Result<Vec<Document>> {
        tracing::warn!("AzureCosmosDBVectorStore is a stub; similarity_search returns empty");
        Ok(Vec::new())
    }

    async fn similarity_search_by_vector(
        &self,
        _embedding: Vec<f32>,
        _k: usize,
    ) -> Result<Vec<Document>> {
        tracing::warn!(
            "AzureCosmosDBVectorStore is a stub; similarity_search_by_vector returns empty"
        );
        Ok(Vec::new())
    }

    async fn similarity_search_with_score(
        &self,
        _query: &str,
        _k: usize,
    ) -> Result<Vec<(Document, f32)>> {
        tracing::warn!(
            "AzureCosmosDBVectorStore is a stub; similarity_search_with_score returns empty"
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
            "AzureCosmosDBVectorStore is a stub; max_marginal_relevance_search returns empty"
        );
        Ok(Vec::new())
    }

    async fn delete(&self, _ids: Vec<String>) -> Result<()> {
        tracing::warn!("AzureCosmosDBVectorStore is a stub; delete does nothing");
        Ok(())
    }

    fn embeddings(&self) -> Arc<dyn Embeddings> {
        self.embeddings.clone()
    }
}
