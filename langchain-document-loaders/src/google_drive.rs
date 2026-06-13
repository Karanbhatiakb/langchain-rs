//! Google Drive document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct GoogleDriveLoader {
    api_key: String,
    client: Client,
}

impl GoogleDriveLoader {
    pub fn new() -> Self {
        let api_key = std::env::var("GOOGLE_DRIVE_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .expect("GOOGLE_DRIVE_API_KEY or GOOGLE_API_KEY environment variable is required");
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    async fn api_get(&self, endpoint: &str) -> Result<String> {
        let sep = if endpoint.contains('?') { "&" } else { "?" };
        let url = format!(
            "https://www.googleapis.com/drive/v3{}{}key={}",
            endpoint, sep, self.api_key
        );
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Google Drive API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Google Drive API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Google Drive API response: {}", e)))
    }

    pub async fn list_files(&self, query: Option<&str>) -> Result<Vec<Document>> {
        let q = query.unwrap_or("");
        let encoded = url::form_urlencoded::byte_serialize(q.as_bytes()).collect::<String>();
        let endpoint = format!("/files?q={}&pageSize=100&fields=files(id,name,mimeType,createdTime,modifiedTime,size)", encoded);
        let body = self.api_get(&endpoint).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Drive files: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(files) = value.get("files").and_then(|f| f.as_array()) {
            for file in files {
                let name = file.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                let file_id = file.get("id").and_then(|v| v.as_str()).unwrap_or("");

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(format!("gdrive:{}", file_id)));
                metadata.insert("file_id".to_string(), serde_json::Value::String(file_id.to_string()));
                metadata.insert("name".to_string(), serde_json::Value::String(name.to_string()));
                metadata.insert("mime_type".to_string(), file.get("mimeType").cloned().unwrap_or(Value::Null));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("google_drive".to_string()));

                documents.push(Document::new(serde_json::to_string(file).unwrap_or_default()).with_metadata(metadata));
            }
        }

        Ok(documents)
    }

    pub async fn get_file(&self, file_id: &str) -> Result<Document> {
        let endpoint = format!("/files/{}?fields=id,name,mimeType,createdTime,modifiedTime,size,description", file_id);
        let body = self.api_get(&endpoint).await?;
        let file: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Drive file: {}", e)))?;

        let name = file.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
        let content = file.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(format!("gdrive:{}", file_id)));
        metadata.insert("file_id".to_string(), serde_json::Value::String(file_id.to_string()));
        metadata.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("google_drive".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }

    pub async fn download_file(&self, file_id: &str) -> Result<Document> {
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}?alt=media&key={}",
            file_id, self.api_key
        );
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to download file: {}", e)))?;

        let content = response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read downloaded file: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(format!("gdrive:{}", file_id)));
        metadata.insert("file_id".to_string(), serde_json::Value::String(file_id.to_string()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("google_drive".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Document>> {
        let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        let endpoint = format!("/files?q={}&pageSize=100&fields=files(id,name,mimeType,createdTime,modifiedTime)", encoded);
        let body = self.api_get(&endpoint).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Drive search results: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(files) = value.get("files").and_then(|f| f.as_array()) {
            for file in files {
                let name = file.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                let file_id = file.get("id").and_then(|v| v.as_str()).unwrap_or("");

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(format!("gdrive:{}", file_id)));
                metadata.insert("file_id".to_string(), serde_json::Value::String(file_id.to_string()));
                metadata.insert("name".to_string(), serde_json::Value::String(name.to_string()));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("google_drive".to_string()));

                documents.push(Document::new(serde_json::to_string(file).unwrap_or_default()).with_metadata(metadata));
            }
        }

        Ok(documents)
    }
}

impl Default for GoogleDriveLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for GoogleDriveLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        self.list_files(None).await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
