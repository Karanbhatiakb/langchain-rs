//! Core [`VectorStore`] trait for document storage and similarity search.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_embeddings::traits::Embeddings;
use serde_json::Value;

/// Trait for vector databases that store documents and support semantic
/// similarity search.
///
/// Implementors must provide an associated embedding model and support
/// adding documents, similarity search (with scores and by vector), MMR
/// search, and deletion.
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Adds documents to the store and returns their IDs.
    async fn add_documents(&self, docs: Vec<Document>) -> Result<Vec<String>>;
    /// Adds raw text strings with optional metadata and returns IDs.
    async fn add_texts(&self, texts: Vec<String>, metadatas: Option<Vec<HashMap<String, Value>>>) -> Result<Vec<String>>;
    /// Searches for documents similar to the query text.
    async fn similarity_search(&self, query: &str, k: usize) -> Result<Vec<Document>>;
    /// Searches for documents similar to the query, returning documents with
    /// similarity scores.
    async fn similarity_search_with_score(&self, query: &str, k: usize) -> Result<Vec<(Document, f32)>>;
    /// Searches for documents similar to the provided embedding vector.
    async fn similarity_search_by_vector(&self, embedding: Vec<f32>, k: usize) -> Result<Vec<Document>>;
    /// Performs Max Marginal Relevance search for diverse results.
    async fn max_marginal_relevance_search(&self, query: &str, k: usize, fetch_k: usize, lambda_mult: f32) -> Result<Vec<Document>>;
    /// Deletes documents with the given IDs.
    async fn delete(&self, ids: Vec<String>) -> Result<()>;
    /// Returns the embeddings model used by this store.
    fn embeddings(&self) -> Arc<dyn Embeddings>;
}
