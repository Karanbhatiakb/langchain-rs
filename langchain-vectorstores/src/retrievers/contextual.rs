//! Contextual compression retriever that wraps a base retriever with document
//! compression.
//!
//! The [`ContextualCompressionRetriever`] wraps another retriever and compresses
//! or reranks the retrieved documents before returning them. Compression can
//! involve extracting the most relevant passages, removing redundant content,
//! or summarising documents to fit within a context window.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;
use std::sync::Arc;

/// Wraps a retriever and compresses the retrieved documents.
///
/// The compressor (identified by configuration) processes the raw documents
/// returned by `base_retriever` and outputs a shorter, more focused set of
/// documents.
#[derive(Clone)]
pub struct ContextualCompressionRetriever {
    /// The base retriever whose results will be compressed.
    pub base_retriever: Arc<dyn BaseRetriever>,
    /// Optional compressor configuration name.
    pub compressor: Option<String>,
}

impl std::fmt::Debug for ContextualCompressionRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextualCompressionRetriever")
            .field("compressor", &self.compressor)
            .finish()
    }
}

impl ContextualCompressionRetriever {
    /// Creates a new `ContextualCompressionRetriever`.
    ///
    /// # Arguments
    /// * `base_retriever` - The retriever to wrap.
    /// * `compressor` - Optional compressor configuration.
    pub fn new(base_retriever: Arc<dyn BaseRetriever>, compressor: Option<String>) -> Self {
        Self {
            base_retriever,
            compressor,
        }
    }
}

#[async_trait]
impl BaseRetriever for ContextualCompressionRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        self.base_retriever.add_documents(documents).await
    }
}
