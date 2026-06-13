//! Structured query construction.
//!
//! Provides [`StructuredQuery`] for representing a query with optional filter
//! and limit, and the [`QueryConstructor`] trait with a default implementation.

use crate::errors::*;
use serde::{Deserialize, Serialize};

/// A structured query with an optional filter and limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredQuery {
    /// The text query.
    pub query: String,
    /// An optional filter expressed as a JSON value.
    pub filter: Option<serde_json::Value>,
    /// An optional maximum number of results.
    pub limit: Option<usize>,
}

impl StructuredQuery {
    /// Creates a new `StructuredQuery` with the given query text and no
    /// filter or limit.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            filter: None,
            limit: None,
        }
    }

    /// Sets the filter (builder pattern).
    pub fn with_filter(mut self, filter: serde_json::Value) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Sets the limit (builder pattern).
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Trait for constructing a [`StructuredQuery`] from a natural-language
/// question.
pub trait QueryConstructor: Send + Sync {
    /// Constructs a structured query from the given question.
    fn construct(&self, question: &str) -> Result<StructuredQuery>;
}

/// A default query constructor that returns the question as-is with no filter
/// or limit.
#[derive(Debug, Clone)]
pub struct DefaultQueryConstructor;

impl DefaultQueryConstructor {
    /// Creates a new `DefaultQueryConstructor`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultQueryConstructor {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryConstructor for DefaultQueryConstructor {
    fn construct(&self, question: &str) -> Result<StructuredQuery> {
        Ok(StructuredQuery::new(question))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_query_constructor() {
        let constructor = DefaultQueryConstructor::new();
        let query = constructor.construct("What is Rust?").unwrap();
        assert_eq!(query.query, "What is Rust?");
        assert!(query.filter.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_structured_query_builder() {
        let query = StructuredQuery::new("find docs")
            .with_filter(serde_json::json!({"category": "tech"}))
            .with_limit(10);
        assert_eq!(query.query, "find docs");
        assert!(query.filter.is_some());
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_structured_query_serialization() {
        let query = StructuredQuery::new("test").with_limit(5);
        let json = serde_json::to_string(&query).unwrap();
        let deserialized: StructuredQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.query, "test");
        assert_eq!(deserialized.limit, Some(5));
    }

    #[test]
    fn test_structured_query_defaults() {
        let query = StructuredQuery::new("hello");
        assert_eq!(query.query, "hello");
        assert!(query.filter.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_structured_query_with_filter_only() {
        let query = StructuredQuery::new("find docs")
            .with_filter(serde_json::json!({"status": "active"}));
        assert!(query.filter.is_some());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_structured_query_with_all_fields() {
        let query = StructuredQuery::new("search")
            .with_filter(serde_json::json!({"type": "article"}))
            .with_limit(25);
        assert_eq!(query.limit, Some(25));
        assert_eq!(query.filter, Some(serde_json::json!({"type": "article"})));
    }

    #[test]
    fn test_structured_query_serialization_with_filter() {
        let query = StructuredQuery::new("hello")
            .with_filter(serde_json::json!({"field": "value"}))
            .with_limit(10);
        let json = serde_json::to_string(&query).unwrap();
        let deserialized: StructuredQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.query, "hello");
        assert_eq!(deserialized.limit, Some(10));
        assert_eq!(deserialized.filter, Some(serde_json::json!({"field": "value"})));
    }

    #[test]
    fn test_default_query_constructor_new() {
        let c = DefaultQueryConstructor::new();
        assert!(c.construct("q").is_ok());
    }

    #[test]
    fn test_query_constructor_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<DefaultQueryConstructor>();
        assert_sync::<DefaultQueryConstructor>();
    }
}
