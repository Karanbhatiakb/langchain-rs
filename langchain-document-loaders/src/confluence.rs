//! Confluence document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct ConfluenceLoader {
    base_url: String,
    username: String,
    api_token: String,
    space_key: Option<String>,
    client: Client,
}

impl ConfluenceLoader {
    pub fn new() -> Self {
        let base_url = std::env::var("CONFLUENCE_URL")
            .expect("CONFLUENCE_URL environment variable is required");
        let username = std::env::var("CONFLUENCE_USERNAME")
            .unwrap_or_default();
        let api_token = std::env::var("CONFLUENCE_API_TOKEN")
            .expect("CONFLUENCE_API_TOKEN environment variable is required");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username,
            api_token,
            space_key: std::env::var("CONFLUENCE_SPACE_KEY").ok(),
            client: Client::new(),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    pub fn with_api_token(mut self, token: impl Into<String>) -> Self {
        self.api_token = token.into();
        self
    }

    pub fn with_space_key(mut self, space_key: impl Into<String>) -> Self {
        self.space_key = Some(space_key.into());
        self
    }

    async fn api_get(&self, endpoint: &str) -> Result<String> {
        let url = format!("{}/rest/api{}", self.base_url, endpoint);
        use base64::Engine;
        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("{}:{}", self.username, self.api_token),
        );

        let response = self.client.get(&url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Confluence API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Confluence API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Confluence API response: {}", e)))
    }

    pub async fn get_space_pages(&self, space_key: Option<&str>) -> Result<Vec<Document>> {
        let key = space_key.or(self.space_key.as_deref())
            .ok_or_else(|| ChainError::ValidationError("No space_key provided".to_string()))?;

        let pages_body = self.api_get(&format!("/space/{}?expand=pages", key)).await?;
        let _value: Value = serde_json::from_str(&pages_body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Confluence space: {}", e)))?;

        let content_body = self.api_get(&format!("/content?spaceKey={}&expand=body.storage,version&limit=100", key)).await?;
        let content_value: Value = serde_json::from_str(&content_body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Confluence content: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(results) = content_value.get("results").and_then(|r| r.as_array()) {
            for page in results {
                let doc = self.page_to_document(page)?;
                documents.push(doc);
            }
        }

        Ok(documents)
    }

    pub async fn get_page(&self, page_id: &str) -> Result<Document> {
        let body = self.api_get(&format!("/content/{}?expand=body.storage,version", page_id)).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Confluence page: {}", e)))?;

        self.page_to_document(&value)
    }

    fn page_to_document(&self, page: &Value) -> Result<Document> {
        let id = page.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let title = page.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
        let content = page.pointer("/body/storage/value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(
            format!("{}/spaces/viewpage.action?pageId={}", self.base_url, id)
        ));
        metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
        metadata.insert("page_id".to_string(), serde_json::Value::String(id.to_string()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("confluence".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }
}

impl Default for ConfluenceLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for ConfluenceLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        self.get_space_pages(None).await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
