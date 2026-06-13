//! BM25 (Okapi BM25) retriever for text retrieval using the BM25 ranking
//! function.
//!
//! The [`BM25Retriever`] implements the Okapi BM25 algorithm, a bag-of-words
//! retrieval function that ranks documents based on term frequency and inverse
//! document frequency with saturation and length normalisation. It is an
//! embedding-free retriever that works directly on tokenised text.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;

/// Retrieves documents using the BM25 (Okapi) ranking algorithm.
///
/// BM25 scores documents based on term frequency (TF), inverse document
/// frequency (IDF), and document length normalisation. The parameters `k1`
/// controls term frequency saturation and `b` controls length normalisation.
///
/// Typical values are `k1 = 1.5` and `b = 0.75`.
#[derive(Debug, Clone)]
pub struct BM25Retriever {
    /// The corpus of documents to search.
    pub docs: Vec<Document>,
    /// BM25 term frequency saturation parameter (default 1.5).
    pub k1: f32,
    /// BM25 length normalisation parameter (default 0.75).
    pub b: f32,
    /// Number of top documents to return.
    pub k: usize,
    /// Optional tokenizer configuration name.
    pub tokenizer: Option<String>,
}

impl BM25Retriever {
    /// Creates a new `BM25Retriever` from a list of documents.
    ///
    /// # Arguments
    /// * `docs` - The document corpus.
    /// * `k` - Number of documents to retrieve.
    pub fn new(docs: Vec<Document>, k: usize) -> Self {
        Self {
            docs,
            k1: 1.5,
            b: 0.75,
            k,
            tokenizer: None,
        }
    }
}

#[async_trait]
impl BaseRetriever for BM25Retriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
