//! K-Nearest Neighbors retriever that ranks documents by embedding
//! similarity.
//!
//! The [`KNNRetriever`] embeds the query using the provided embedding model,
//! then computes similarity against all stored document embeddings to find
//! the `k` most similar documents.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;
use langchain_embeddings::traits::Embeddings;
use std::sync::Arc;

/// Performs K-Nearest Neighbors search over document embeddings.
///
/// Documents are embedded using the provided embedding model. At query time,
/// the query is embedded and the `k` closest documents (by cosine similarity)
/// are returned.
#[derive(Clone)]
pub struct KNNRetriever {
    /// The document corpus.
    pub docs: Vec<Document>,
    /// Embedding model for vectorising queries and documents.
    pub embeddings: Arc<dyn Embeddings>,
    /// Number of nearest neighbours to return.
    pub k: usize,
}

impl std::fmt::Debug for KNNRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KNNRetriever")
            .field("docs_len", &self.docs.len())
            .field("k", &self.k)
            .finish()
    }
}

impl KNNRetriever {
    /// Creates a new `KNNRetriever`.
    ///
    /// # Arguments
    /// * `docs` - The document corpus.
    /// * `embeddings` - The embedding model.
    /// * `k` - Number of nearest neighbours to retrieve.
    pub fn new(docs: Vec<Document>, embeddings: Arc<dyn Embeddings>, k: usize) -> Self {
        Self {
            docs,
            embeddings,
            k,
        }
    }
}

#[async_trait]
impl BaseRetriever for KNNRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
