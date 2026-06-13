//! iFrame document loader.
//!
//! Loads the content of an `<iframe>` by fetching its `src` URL.
//! Useful when you have a page that embeds content via iframes and you
//! need to extract that embedded content.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads content from an iframe source URL.
#[derive(Debug, Clone)]
pub struct IFrameLoader {
    src_url: String,
    client: Client,
}

impl IFrameLoader {
    /// Create a new `IFrameLoader`.
    pub fn new(src_url: impl Into<String>) -> Self {
        Self {
            src_url: src_url.into(),
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
impl BaseLoader for IFrameLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self.client
            .get(&self.src_url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to fetch iframe source '{}': {}", self.src_url, e)))?;

        let content = response.text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read iframe response body: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.src_url.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("iframe".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
