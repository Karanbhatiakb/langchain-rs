//! Web page document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct WebBaseLoader {
    urls: Vec<String>,
    client: Client,
}

impl WebBaseLoader {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            urls,
            client: Client::new(),
        }
    }

    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }
}

#[async_trait]
impl BaseLoader for WebBaseLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        for url in &self.urls {
            let response = self.client
                .get(url)
                .send()
                .await
                .map_err(|e| ChainError::IOError(format!("Failed to fetch URL '{}': {}", url, e)))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ChainError::IOError(format!(
                    "HTTP {} when fetching '{}'", status, url
                )));
            }

            let content = response.text()
                .await
                .map_err(|e| ChainError::IOError(format!("Failed to read response body from '{}': {}", url, e)))?;

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(url.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("web".to_string()));

            documents.push(Document::new(content).with_metadata(metadata));
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

pub struct SitemapLoader {
    sitemap_url: String,
    client: Client,
}

impl SitemapLoader {
    pub fn new(sitemap_url: impl Into<String>) -> Self {
        Self {
            sitemap_url: sitemap_url.into(),
            client: Client::new(),
        }
    }

    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    pub fn parse_sitemap_urls(xml: &str) -> Vec<String> {
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
            return Err(ChainError::IOError(format!(
                "No URLs found in sitemap '{}'", self.sitemap_url
            )));
        }

        let web_loader = WebBaseLoader::new(urls).with_client(self.client.clone());
        web_loader.load().await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}

pub struct URLLoader {
    url: String,
    client: Client,
}

impl URLLoader {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: Client::new(),
        }
    }

    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }
}

#[async_trait]
impl BaseLoader for URLLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self.client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to fetch URL '{}': {}", self.url, e)))?;

        let content = response.text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read response body from '{}': {}", self.url, e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.url.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("url".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
