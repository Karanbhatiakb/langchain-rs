//! Sitemap document loader.
//!
//! Downloads an XML sitemap, extracts page URLs, and fetches each page as a
//! separate document.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads documents by scraping URLs listed in an XML sitemap.
#[derive(Debug, Clone)]
pub struct SitemapLoader {
    sitemap_url: String,
    client: Client,
}

impl SitemapLoader {
    /// Create a new `SitemapLoader`.
    pub fn new(sitemap_url: impl Into<String>) -> Self {
        Self {
            sitemap_url: sitemap_url.into(),
            client: Client::new(),
        }
    }

    /// Override the HTTP client.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Extract `<loc>` URLs from the sitemap XML.
    fn parse_sitemap_urls(xml: &str) -> Vec<String> {
        let mut urls = Vec::new();
        for line in xml.lines() {
            let trimmed = line.trim();
            if let Some(start) = trimmed.find("<loc>") {
                if let Some(end) = trimmed.find("</loc>") {
                    let url = &trimmed[start + 5..end];
                    urls.push(url.to_string());
                }
            }
        }
        urls
    }
}

#[async_trait]
impl BaseLoader for SitemapLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self.client
            .get(&self.sitemap_url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to fetch sitemap '{}': {}", self.sitemap_url, e)))?;

        let xml = response.text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read sitemap response: {}", e)))?;

        let urls = Self::parse_sitemap_urls(&xml);

        if urls.is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.sitemap_url.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("sitemap".to_string()));
            return Ok(vec![Document::new(xml).with_metadata(metadata)]);
        }

        let mut documents = Vec::new();
        for url in urls {
            match self.client.get(&url).send().await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok(content) => {
                            let mut metadata = HashMap::new();
                            metadata.insert("source".to_string(), serde_json::Value::String(url));
                            metadata.insert("loader_type".to_string(), serde_json::Value::String("sitemap".to_string()));
                            documents.push(Document::new(content).with_metadata(metadata));
                        }
                        Err(e) => {
                            let msg = format!("Failed to read page content: {}", e);
                            let mut metadata = HashMap::new();
                            metadata.insert("loader_type".to_string(), serde_json::Value::String("sitemap".to_string()));
                            documents.push(Document::new(msg).with_metadata(metadata));
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Failed to fetch URL '{}': {}", url, e);
                    let mut metadata = HashMap::new();
                    metadata.insert("loader_type".to_string(), serde_json::Value::String("sitemap".to_string()));
                    documents.push(Document::new(msg).with_metadata(metadata));
                }
            }
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
