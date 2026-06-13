//! Microsoft OneDrive document loader.
//!
//! Fetches files from OneDrive using the Microsoft Graph API.
//! This is a stub implementation — provide your own access token and
//! configure the API endpoint as needed.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads documents from Microsoft OneDrive via the Graph API.
#[derive(Debug, Clone)]
pub struct OneDriveLoader {
    access_token: String,
    query: String,
    client: Client,
}

impl OneDriveLoader {
    /// Create a new `OneDriveLoader` with the given access token and query.
    pub fn new(access_token: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            query: query.into(),
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
impl BaseLoader for OneDriveLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let url = format!("https://graph.microsoft.com/v1.0/me/drive/search(q='{}')?select=name,webUrl", self.query);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("OneDrive API request failed: {}", e)))?;

        let body = response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read OneDrive response: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("loader_type".to_string(), serde_json::Value::String("onedrive".to_string()));
        metadata.insert("api_response".to_string(), serde_json::Value::String(body));

        Ok(vec![Document::new("OneDrive document stub — replace with actual file content").with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
