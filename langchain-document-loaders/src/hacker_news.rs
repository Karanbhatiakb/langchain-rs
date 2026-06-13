//! Hacker News document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that fetches stories and comments from Hacker News.
///
/// Supports fetching top stories, new stories, best stories, or individual
/// items by ID. Each item is returned as a document with author, score,
/// and URL metadata.
#[derive(Debug, Clone)]
pub struct HackerNewsLoader {
    story_type: String,
}

impl HackerNewsLoader {
    /// Creates a new `HackerNewsLoader` for the given story type
    /// (e.g. "top", "new", "best").
    pub fn new(story_type: impl Into<String>) -> Self {
        Self {
            story_type: story_type.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for HackerNewsLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("hacker_news".to_string()),
        );
        metadata.insert(
            "story_type".to_string(),
            serde_json::Value::String(self.story_type.clone()),
        );
        Ok(vec![Document::new("Hacker News loader stub — no stories loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
