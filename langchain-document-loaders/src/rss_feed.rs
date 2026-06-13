//! RSS Feed document loader.
//!
//! Fetches an RSS/Atom feed, parses items, and returns each item as a separate
//! document.  Basic XML parsing is done inline — add a dedicated RSS crate for
//! production use.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads documents from an RSS or Atom feed.
#[derive(Debug, Clone)]
pub struct RSSFeedLoader {
    feed_url: String,
    client: Client,
}

impl RSSFeedLoader {
    /// Create a new `RSSFeedLoader` for the given feed URL.
    pub fn new(feed_url: impl Into<String>) -> Self {
        Self {
            feed_url: feed_url.into(),
            client: Client::new(),
        }
    }

    /// Override the HTTP client.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Extract `<item>` / `<entry>` titles from RSS / Atom XML.
    fn parse_items(xml: &str) -> Vec<String> {
        let mut items = Vec::new();
        for line in xml.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("<title>") && trimmed.ends_with("</title>") {
                let inner = &trimmed[7..trimmed.len() - 8];
                if !inner.is_empty() {
                    items.push(inner.to_string());
                }
            }
        }
        items
    }
}

#[async_trait]
impl BaseLoader for RSSFeedLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self.client
            .get(&self.feed_url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to fetch RSS feed '{}': {}", self.feed_url, e)))?;

        let xml = response.text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read RSS feed response: {}", e)))?;

        let titles = Self::parse_items(&xml);
        let mut documents = Vec::new();

        for title in titles {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.feed_url.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("rss_feed".to_string()));
            documents.push(Document::new(title).with_metadata(metadata));
        }

        if documents.is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.feed_url.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("rss_feed".to_string()));
            documents.push(Document::new(xml).with_metadata(metadata));
        }

        Ok(documents)
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
