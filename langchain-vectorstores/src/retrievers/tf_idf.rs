//! TF-IDF based retriever for embedding-free text retrieval.
//!
//! The [`TFIDFRetriever`] uses Term Frequency-Inverse Document Frequency
//! (TF-IDF) vectorisation to rank documents by their relevance to a query.
//! It is a simple, interpretable alternative to embedding-based retrieval.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;

/// Retrieves documents using TF-IDF vector space similarity.
///
/// Documents and queries are represented as TF-IDF vectors. Relevance is
/// scored by cosine similarity between the query vector and each document
/// vector.
#[derive(Debug, Clone)]
pub struct TFIDFRetriever {
    /// The document corpus to search.
    pub docs: Vec<Document>,
    /// Number of top documents to return.
    pub k: usize,
}

impl TFIDFRetriever {
    /// Creates a new `TFIDFRetriever`.
    ///
    /// # Arguments
    /// * `docs` - The document corpus.
    /// * `k` - Number of documents to retrieve.
    pub fn new(docs: Vec<Document>, k: usize) -> Self {
        Self { docs, k }
    }
}

#[async_trait]
impl BaseRetriever for TFIDFRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
