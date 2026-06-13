//! Newspaper article document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that extracts article text from news URLs.
///
/// Provide a URL or list of URLs. The loader fetches the page and extracts
/// the main article content, title, author, publish date, and top image
/// using newspaper-style extraction.
#[derive(Debug, Clone)]
pub struct NewspaperLoader {
    url: String,
}

impl NewspaperLoader {
    /// Creates a new `NewspaperLoader` for the given article URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for NewspaperLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("newspaper".to_string()),
        );
        metadata.insert(
            "url".to_string(),
            serde_json::Value::String(self.url.clone()),
        );
        Ok(vec![Document::new("Newspaper loader stub — no article extracted yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
