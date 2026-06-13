//! Document type used throughout LangChain for representing text chunks
//! with associated metadata and optional relevance scores.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A text document with associated metadata and optional relevance score.
///
/// Documents are the fundamental data unit in LangChain — they flow through
/// splitters, retrievers, vector stores, chains, and agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The text content of the document.
    pub page_content: String,
    /// Arbitrary metadata key-value pairs attached to the document.
    pub metadata: HashMap<String, serde_json::Value>,
    /// An optional relevance score (e.g. from similarity search).
    pub score: Option<f32>,
}

impl Document {
    /// Creates a new `Document` with the given page content.
    ///
    /// `metadata` starts empty and `score` is `None`.
    pub fn new(page_content: impl Into<String>) -> Self {
        Self {
            page_content: page_content.into(),
            metadata: HashMap::new(),
            score: None,
        }
    }

    /// Sets the `metadata` field and returns `self` (builder pattern).
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the `score` field and returns `self`.
    pub fn with_score(mut self, score: f32) -> Self {
        self.score = Some(score);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_document_new() {
        let doc = Document::new("test content");
        assert_eq!(doc.page_content, "test content");
        assert!(doc.metadata.is_empty());
        assert!(doc.score.is_none());
    }

    #[test]
    fn test_document_new_empty() {
        let doc = Document::new("");
        assert_eq!(doc.page_content, "");
    }

    #[test]
    fn test_document_with_metadata() {
        let mut meta = HashMap::new();
        meta.insert("source".into(), serde_json::Value::String("web".into()));
        meta.insert("id".into(), serde_json::Value::Number(42.into()));
        let doc = Document::new("content").with_metadata(meta);
        assert_eq!(doc.metadata.get("source").unwrap(), "web");
        assert_eq!(doc.metadata.get("id").unwrap(), 42);
    }

    #[test]
    fn test_document_with_score() {
        let doc = Document::new("content").with_score(0.95);
        assert_eq!(doc.score, Some(0.95));
    }

    #[test]
    fn test_document_score_none() {
        let doc = Document::new("content");
        assert!(doc.score.is_none());
    }

    #[test]
    fn test_document_clone() {
        let mut meta = HashMap::new();
        meta.insert("k".into(), serde_json::Value::String("v".into()));
        let doc = Document::new("text").with_metadata(meta).with_score(0.5);
        let cloned = doc.clone();
        assert_eq!(cloned.page_content, "text");
        assert_eq!(cloned.score, Some(0.5));
        assert_eq!(cloned.metadata.get("k").unwrap(), "v");
    }

    #[test]
    fn test_document_serde() {
        let doc = Document::new("hello");
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.page_content, "hello");
    }

    #[test]
    fn test_document_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Document>();
        assert_sync::<Document>();
    }
}
