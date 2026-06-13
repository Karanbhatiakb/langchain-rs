//! News URL document loader.
//!
//! Fetches a news article from a given URL and returns its content.
//! This loader can be extended to integrate with news-specific APIs.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads a news article from a URL.
#[derive(Debug, Clone)]
pub struct NewsLoader {
    url: String,
    client: Client,
}

impl NewsLoader {
    /// Create a new `NewsLoader`.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: Client::new(),
        }
    }

    /// Override the HTTP client.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }
}

#[async_trait]
impl BaseLoader for NewsLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self.client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to fetch news URL '{}': {}", self.url, e)))?;

        let content = response.text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read news response body: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.url.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("news".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
