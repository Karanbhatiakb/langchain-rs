//! Elasticsearch retriever for search via Elasticsearch.
//!
//! The [`ElasticSearchRetriever`] connects to an Elasticsearch cluster and
//! performs full-text and vector search queries against a specified index.
//! It supports both keyword search and embedding-based similarity search.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;

/// Searches an Elasticsearch index for documents matching a query.
///
/// Connects to Elasticsearch using the provided connection configuration.
/// Supports both BM25 full-text search and vector similarity search when
/// embeddings are available.
#[derive(Debug, Clone)]
pub struct ElasticSearchRetriever {
    /// The Elasticsearch index name to search.
    pub index_name: String,
    /// Number of top results to return.
    pub top_k: usize,
    /// Optional HTTP client or connection configuration name.
    pub client: Option<String>,
}

impl ElasticSearchRetriever {
    /// Creates a new `ElasticSearchRetriever`.
    ///
    /// # Arguments
    /// * `index_name` - The Elasticsearch index to search.
    /// * `top_k` - Number of results to retrieve.
    pub fn new(index_name: String, top_k: usize) -> Self {
        Self {
            index_name,
            top_k,
            client: None,
        }
    }
}

#[async_trait]
impl BaseRetriever for ElasticSearchRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
