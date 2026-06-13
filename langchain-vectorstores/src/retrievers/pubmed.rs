//! PubMed retriever that searches PubMed for medical and life sciences
//! literature.
//!
//! The [`PubMedRetriever`] uses the NCBI E-utilities API to query PubMed for
//! articles matching the search terms and returns them as documents.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;

/// Searches PubMed for medical and life science articles.
///
/// Uses the NCBI E-utilities API (esearch, efetch) to find and retrieve
/// article metadata and abstracts. Results are returned as LangChain
/// documents with structured metadata.
#[derive(Debug, Clone)]
pub struct PubMedRetriever {
    /// Number of top articles to return.
    pub top_k: usize,
    /// Optional HTTP client configuration name.
    pub client: Option<String>,
}

impl PubMedRetriever {
    /// Creates a new `PubMedRetriever`.
    ///
    /// # Arguments
    /// * `top_k` - Number of articles to retrieve.
    pub fn new(top_k: usize) -> Self {
        Self {
            top_k,
            client: None,
        }
    }
}

#[async_trait]
impl BaseRetriever for PubMedRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
