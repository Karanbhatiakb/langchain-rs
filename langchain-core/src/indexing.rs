//! Document indexing utilities.
//!
//! Provides [`IndexingResult`] and the [`index_documents`] function for adding
//! documents to a vector store and tracking indexing statistics.

use crate::documents::Document;
use crate::errors::*;
use crate::retrievers::VectorStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Statistics returned by an indexing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingResult {
    /// Number of documents newly added.
    pub num_added: usize,
    /// Number of documents updated (existing documents that changed).
    pub num_updated: usize,
    /// Number of documents deleted.
    pub num_deleted: usize,
    /// Number of documents skipped (already up-to-date).
    pub num_skipped: usize,
}

/// Indexes the given documents into the vector store.
///
/// Currently this simply adds all documents and reports `num_added` equal to
/// the document count. In the future this will support deduplication and
/// updates based on `doc_id_key`.
pub async fn index_documents(
    vectorstore: Arc<dyn VectorStore>,
    documents: Vec<Document>,
    _doc_id_key: &str,
) -> Result<IndexingResult> {
    let num_added = documents.len();
    vectorstore.add_documents(documents)?;
    Ok(IndexingResult {
        num_added,
        num_updated: 0,
        num_deleted: 0,
        num_skipped: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockVectorStore;

    impl VectorStore for MockVectorStore {
        fn similarity_search(&self, _query: &str, _k: usize) -> crate::errors::Result<Vec<Document>> {
            Ok(Vec::new())
        }
        fn add_documents(&self, docs: Vec<Document>) -> crate::errors::Result<Vec<String>> {
            Ok(docs.iter().map(|_| "mock_id".to_string()).collect())
        }
    }

    #[tokio::test]
    async fn test_indexing_result_defaults() {
        let result = IndexingResult {
            num_added: 5,
            num_updated: 2,
            num_deleted: 1,
            num_skipped: 3,
        };
        assert_eq!(result.num_added, 5);
        assert_eq!(result.num_updated, 2);
        assert_eq!(result.num_deleted, 1);
        assert_eq!(result.num_skipped, 3);
    }

    #[tokio::test]
    async fn test_indexing_result_serialization() {
        let result = IndexingResult {
            num_added: 10,
            num_updated: 0,
            num_deleted: 0,
            num_skipped: 0,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: IndexingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.num_added, deserialized.num_added);
    }

    #[tokio::test]
    async fn test_index_documents_adds_all() {
        let store: Arc<dyn VectorStore> = Arc::new(MockVectorStore);
        let docs = vec![Document::new("doc1"), Document::new("doc2")];
        let result = index_documents(store, docs, "id_field").await.unwrap();
        assert_eq!(result.num_added, 2);
        assert_eq!(result.num_updated, 0);
        assert_eq!(result.num_deleted, 0);
        assert_eq!(result.num_skipped, 0);
    }

    #[tokio::test]
    async fn test_index_documents_empty() {
        let store: Arc<dyn VectorStore> = Arc::new(MockVectorStore);
        let docs = vec![];
        let result = index_documents(store, docs, "id_field").await.unwrap();
        assert_eq!(result.num_added, 0);
    }

    #[tokio::test]
    async fn test_indexing_result_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<IndexingResult>();
        assert_sync::<IndexingResult>();
    }
}
