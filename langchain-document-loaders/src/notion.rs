//! Notion document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct NotionLoader {
    token: String,
    database_id: Option<String>,
    client: Client,
}

impl NotionLoader {
    pub fn new() -> Self {
        let token = std::env::var("NOTION_TOKEN")
            .expect("NOTION_TOKEN environment variable is required for NotionLoader");
        Self {
            token,
            database_id: std::env::var("NOTION_DATABASE_ID").ok(),
            client: Client::new(),
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = token.into();
        self
    }

    pub fn with_database_id(mut self, database_id: impl Into<String>) -> Self {
        self.database_id = Some(database_id.into());
        self
    }

    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    async fn notion_get(&self, endpoint: &str) -> Result<String> {
        let url = format!("https://api.notion.com/v1{}", endpoint);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", "2022-06-28")
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Notion API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Notion API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Notion API response: {}", e)))
    }

    async fn notion_post(&self, endpoint: &str, body: &Value) -> Result<String> {
        let url = format!("https://api.notion.com/v1{}", endpoint);
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", "2022-06-28")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Notion API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Notion API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Notion API response: {}", e)))
    }

    pub async fn list_databases(&self) -> Result<Vec<Document>> {
        let body = self.notion_get("/databases").await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Notion databases: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(results) = value.get("results").and_then(|r| r.as_array()) {
            for db in results {
                let id = db.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
                let title = db.pointer("/title/0/plain_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled");

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(format!("notion_db:{}", id)));
                metadata.insert("database_id".to_string(), serde_json::Value::String(id.to_string()));
                metadata.insert("title".to_string(), serde_json::Value::String(title.to_string()));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("notion".to_string()));

                documents.push(Document::new(serde_json::to_string(db).unwrap_or_default()).with_metadata(metadata));
            }
        }

        Ok(documents)
    }

    pub async fn query_database(&self, database_id: Option<&str>) -> Result<Vec<Document>> {
        let db_id = database_id.or(self.database_id.as_deref())
            .ok_or_else(|| ChainError::ValidationError("No database_id provided".to_string()))?;

        let body = self.notion_post(&format!("/databases/{}/query", db_id), &serde_json::json!({})).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Notion query results: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(results) = value.get("results").and_then(|r| r.as_array()) {
            for page in results {
                let id = page.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
                let content = extract_notion_page_text(page);

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(format!("notion_page:{}", id)));
                metadata.insert("page_id".to_string(), serde_json::Value::String(id.to_string()));
                metadata.insert("database_id".to_string(), serde_json::Value::String(db_id.to_string()));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("notion".to_string()));

                documents.push(Document::new(content).with_metadata(metadata));
            }
        }

        Ok(documents)
    }

    pub async fn get_page_blobs(&self, page_id: &str) -> Result<Document> {
        let body = self.notion_get(&format!("/pages/{}", page_id)).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Notion page: {}", e)))?;

        let content = extract_notion_page_text(&value);

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(format!("notion_page:{}", page_id)));
        metadata.insert("page_id".to_string(), serde_json::Value::String(page_id.to_string()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("notion".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }
}

fn extract_notion_page_text(page: &Value) -> String {
    let mut texts = Vec::new();
    if let Some(properties) = page.get("properties").and_then(|p| p.as_object()) {
        for (_key, prop) in properties {
            if let Some(title) = prop.get("title").and_then(|t| t.as_array()) {
                for t in title {
                    if let Some(text) = t.get("plain_text").and_then(|v| v.as_str()) {
                        texts.push(text.to_string());
                    }
                }
            }
            if let Some(rich_text) = prop.get("rich_text").and_then(|r| r.as_array()) {
                for t in rich_text {
                    if let Some(text) = t.get("plain_text").and_then(|v| v.as_str()) {
                        texts.push(text.to_string());
                    }
                }
            }
        }
    }
    texts.join("\n")
}

#[async_trait]
impl BaseLoader for NotionLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        if let Some(db_id) = &self.database_id {
            self.query_database(Some(db_id)).await
        } else {
            self.list_databases().await
        }
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}

impl Default for NotionLoader {
    fn default() -> Self {
        Self::new()
    }
}
