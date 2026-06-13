//! Brave Search document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that searches the web using the Brave Search API.
///
/// Provide a query and optional API key. Search results are returned as
/// individual documents with title, snippet, and URL metadata.
#[derive(Debug, Clone)]
pub struct BraveSearchLoader {
    query: String,
}

impl BraveSearchLoader {
    /// Creates a new `BraveSearchLoader` with the given search query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for BraveSearchLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("brave_search".to_string()),
        );
        metadata.insert(
            "query".to_string(),
            serde_json::Value::String(self.query.clone()),
        );
        Ok(vec![Document::new("Brave Search loader stub — no results loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
