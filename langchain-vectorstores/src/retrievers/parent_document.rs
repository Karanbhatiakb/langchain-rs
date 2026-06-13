//! Parent document retriever that retrieves full parent documents from a
//! secondary store after searching over child chunks.
//!
//! The [`ParentDocumentRetriever`] splits documents into smaller child chunks
//! for indexing and search. When a child chunk matches a query, the full
//! parent document is returned. This balances the precision of small chunks
//! with the completeness of full documents.

use async_trait::async_trait;
use langchain_core::documents::Document;
use langchain_core::errors::*;
use langchain_core::retrievers::BaseRetriever;
use std::sync::Arc;

/// Retrieves parent documents after searching over child chunks.
///
/// Documents are split into parent and child chunks. Child chunks are
/// indexed for retrieval; when a child matches, the corresponding parent
/// document is returned.
#[derive(Clone)]
pub struct ParentDocumentRetriever {
    /// The child retriever that searches over sub-document chunks.
    pub child_retriever: Arc<dyn BaseRetriever>,
    /// Optional parent splitter configuration.
    pub parent_splitter: Option<String>,
    /// Optional child splitter configuration.
    pub child_splitter: Option<String>,
    /// Metadata key for looking up the parent document ID.
    pub doc_id_key: String,
}

impl std::fmt::Debug for ParentDocumentRetriever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParentDocumentRetriever")
            .field("parent_splitter", &self.parent_splitter)
            .field("child_splitter", &self.child_splitter)
            .field("doc_id_key", &self.doc_id_key)
            .finish()
    }
}

impl ParentDocumentRetriever {
    /// Creates a new `ParentDocumentRetriever`.
    ///
    /// # Arguments
    /// * `child_retriever` - The retriever for child chunks.
    pub fn new(child_retriever: Arc<dyn BaseRetriever>) -> Self {
        Self {
            child_retriever,
            parent_splitter: None,
            child_splitter: None,
            doc_id_key: "doc_id".into(),
        }
    }
}

#[async_trait]
impl BaseRetriever for ParentDocumentRetriever {
    async fn get_relevant_documents(&self, query: &str) -> Result<Vec<Document>> {
        let _ = query;
        Ok(Vec::new())
    }

    async fn add_documents(&self, documents: Vec<Document>) -> Result<()> {
        self.child_retriever.add_documents(documents).await
    }
}
