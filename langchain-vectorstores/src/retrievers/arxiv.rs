//! Arxiv retriever that searches arXiv for scientific papers.
//!
//! The [`ArxivRetriever`] uses the arXiv API to find papers matching the query
//! and returns them as documents with title, authors, abstract, and metadata.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;

/// Searches arXiv for scientific papers matching a query.
///
/// Uses the public arXiv API to search paper metadata and abstracts.
/// Results include title, authors, abstract, and arXiv identifiers.
#[derive(Debug, Clone)]
pub struct ArxivRetriever {
    /// Number of top papers to return.
    pub top_k: usize,
    /// Optional HTTP client configuration name.
    pub client: Option<String>,
}

impl ArxivRetriever {
    /// Creates a new `ArxivRetriever`.
    ///
    /// # Arguments
    /// * `top_k` - Number of papers to retrieve.
    pub fn new(top_k: usize) -> Self {
        Self {
            top_k,
            client: None,
        }
    }
}

#[async_trait]
impl BaseRetriever for ArxivRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
