//! GitBook document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct GitBookLoader {
    space_id: String,
    api_token: String,
    client: Client,
}

impl GitBookLoader {
    pub fn new(space_id: impl Into<String>) -> Self {
        let api_token = std::env::var("GITBOOK_API_TOKEN")
            .expect("GITBOOK_API_TOKEN environment variable is required");
        Self {
            space_id: space_id.into(),
            api_token,
            client: Client::new(),
        }
    }

    pub fn with_api_token(mut self, token: impl Into<String>) -> Self {
        self.api_token = token.into();
        self
    }

    async fn api_get(&self, endpoint: &str) -> Result<String> {
        let url = format!("https://api.gitbook.com/v1{}", endpoint);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("GitBook API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "GitBook API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read GitBook API response: {}", e)))
    }

    pub async fn list_pages(&self) -> Result<Vec<Document>> {
        let body = self.api_get(&format!("/spaces/{}/content", self.space_id)).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse GitBook pages: {}", e)))?;

        let mut documents = Vec::new();
        let mut stack = Vec::new();

        if let Some(pages) = value.get("pages").and_then(|p| p.as_array()) {
            for page in pages {
                stack.push(page.clone());
            }
        }

        while let Some(page) = stack.pop() {
            let title = page.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
            let path = page.get("path").and_then(|v| v.as_str()).unwrap_or("/");
            let id = page.get("id").and_then(|v| v.as_str()).unwrap_or("");

            if let Some(children) = page.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    stack.push(child.clone());
                }
            }

            let page_body = self.api_get(&format!("/spaces/{}/content/page/{}", self.space_id, id)).await;
            let content = match page_body {
                Ok(body) => {
                    let page_value: Value = serde_json::from_str(&body).unwrap_or_default();
                    page_value.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string()
                }
                Err(_) => String::new(),
            };

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(
                format!("https://app.gitbook.com/o/-/s/{}/{}", self.space_id, path)
            ));
            metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
            metadata.insert("path".to_string(), serde_json::Value::String(path.to_string()));
            metadata.insert("space_id".to_string(), serde_json::Value::String(self.space_id.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("gitbook".to_string()));

            documents.push(Document::new(content).with_metadata(metadata));
        }

        Ok(documents)
    }

    pub async fn get_page(&self, page_id: &str) -> Result<Document> {
        let body = self.api_get(&format!("/spaces/{}/content/page/{}", self.space_id, page_id)).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse GitBook page: {}", e)))?;

        let title = value.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
        let path = value.get("path").and_then(|v| v.as_str()).unwrap_or("/");
        let content = value.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(
            format!("https://app.gitbook.com/o/-/s/{}/{}", self.space_id, path)
        ));
        metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
        metadata.insert("path".to_string(), serde_json::Value::String(path.to_string()));
        metadata.insert("space_id".to_string(), serde_json::Value::String(self.space_id.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("gitbook".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }
}

#[async_trait]
impl BaseLoader for GitBookLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        self.list_pages().await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
