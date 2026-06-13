//! Playwright URL document loader.
//!
//! Controls a Playwright browser via its CDP / WebSocket endpoint to
//! retrieve rendered page content.  Requires a running Playwright server.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads a web page via Playwright for JavaScript-rendered content.
#[derive(Debug, Clone)]
pub struct PlaywrightURLLoader {
    url: String,
    playwright_url: String,
    client: Client,
}

impl PlaywrightURLLoader {
    /// Create a new `PlaywrightURLLoader`.
    ///
    /// The Playwright server URL defaults to `http://localhost:3000`.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            playwright_url: "http://localhost:3000".to_string(),
            client: Client::new(),
        }
    }

    /// Override the Playwright server URL.
    pub fn with_playwright_url(mut self, playwright_url: impl Into<String>) -> Self {
        self.playwright_url = playwright_url.into();
        self
    }
}

#[async_trait]
impl BaseLoader for PlaywrightURLLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let payload = serde_json::json!({
            "url": self.url,
            "waitUntil": "networkidle"
        });

        let resp = self.client
            .post(format!("{}/load", self.playwright_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Playwright request failed: {}", e)))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| ChainError::IOError(format!("Failed to parse Playwright response: {}", e)))?;

        let content = body["content"]
            .as_str()
            .unwrap_or("");

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.url.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("playwright_url".to_string()));

        Ok(vec![Document::new(content.to_string()).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
