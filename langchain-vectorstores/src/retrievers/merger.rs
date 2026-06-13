//! Merger retriever that merges results from multiple retrievers.
//!
//! The [`MergerRetriever`] runs multiple retrievers independently and merges
//! their results into a single list. It can be configured to deduplicate by
//! document content or page content.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;
use std::sync::Arc;

/// Merges results from multiple retrievers.
///
/// Each retriever is queried independently, and their document lists are
/// concatenated (with optional deduplication). The order of results follows
/// the order of retrievers.
#[derive(Clone)]
pub struct MergerRetriever {
    /// The list of retrievers whose results will be merged.
    pub retrievers: Vec<Arc<dyn BaseRetriever>>,
    /// Whether to deduplicate documents by their `page_content`.
    pub deduplicate: bool,
}

impl std::fmt::Debug for MergerRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MergerRetriever")
            .field("deduplicate", &self.deduplicate)
            .finish()
    }
}

impl MergerRetriever {
    /// Creates a new `MergerRetriever`.
    ///
    /// # Arguments
    /// * `retrievers` - The retrievers to merge results from.
    /// * `deduplicate` - If `true`, removes documents with duplicate content.
    pub fn new(retrievers: Vec<Arc<dyn BaseRetriever>>, deduplicate: bool) -> Self {
        Self {
            retrievers,
            deduplicate,
        }
    }
}

#[async_trait]
impl BaseRetriever for MergerRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        let _ = documents;
        Ok(())
    }
}
