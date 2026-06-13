//! Microsoft SharePoint document loader.
//!
//! Fetches documents from SharePoint sites using the Microsoft Graph API.
//! Stub implementation — override the site and list IDs in production.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads documents from a SharePoint site via the Graph API.
#[derive(Debug, Clone)]
pub struct SharePointLoader {
    access_token: String,
    site_id: String,
    list_id: Option<String>,
    client: Client,
}

impl SharePointLoader {
    /// Create a new `SharePointLoader`.
    pub fn new(access_token: impl Into<String>, site_id: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            site_id: site_id.into(),
            list_id: None,
            client: Client::new(),
        }
    }

    /// Set the document library / list ID.
    pub fn with_list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
        self
    }

    /// Override the HTTP client.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }
}

#[async_trait]
impl BaseLoader for SharePointLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let url = match &self.list_id {
            Some(list_id) => format!(
                "https://graph.microsoft.com/v1.0/sites/{}/lists/{}/items?expand=fields",
                self.site_id, list_id
            ),
            None => format!(
                "https://graph.microsoft.com/v1.0/sites/{}/lists",
                self.site_id
            ),
        };

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("SharePoint API request failed: {}", e)))?;

        let body = response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read SharePoint response: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("loader_type".to_string(), serde_json::Value::String("sharepoint".to_string()));
        metadata.insert("api_response".to_string(), serde_json::Value::String(body));

        Ok(vec![Document::new("SharePoint document stub — replace with actual file content").with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
