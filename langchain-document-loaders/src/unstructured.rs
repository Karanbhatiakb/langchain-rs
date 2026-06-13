//! Unstructured document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct UnstructuredLoader {
    file_path: String,
    api_url: String,
    api_key: Option<String>,
    strategy: String,
    client: Client,
}

impl UnstructuredLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        let api_url = std::env::var("UNSTRUCTURED_API_URL")
            .unwrap_or_else(|_| "https://api.unstructured.io/general/v0/general".to_string());
        let api_key = std::env::var("UNSTRUCTURED_API_KEY").ok();

        Self {
            file_path: file_path.into(),
            api_url,
            api_key,
            strategy: "auto".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = strategy.into();
        self
    }

    async fn process(&self) -> Result<Vec<Document>> {
        let content = tokio::fs::read(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read file '{}': {}", self.file_path, e)))?;

        let file_name = std::path::Path::new(&self.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        let mime_type = mime_guess::from_path(&self.file_path)
            .first_or_octet_stream()
            .to_string();

        let part = reqwest::multipart::Part::bytes(content)
            .file_name(file_name)
            .mime_str(&mime_type)
            .map_err(|e| ChainError::IOError(format!("Failed to create multipart: {}", e)))?;

        let form = reqwest::multipart::Form::new()
            .part("files", part)
            .text("strategy", self.strategy.clone());

        let mut req = self.client.post(&self.api_url).multipart(form);

        if let Some(ref key) = self.api_key {
            req = req.header("unstructured-api-key", key);
        }

        let response = req.send()
            .await
            .map_err(|e| ChainError::IOError(format!("Unstructured API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Unstructured API returned HTTP {}", response.status()
            )));
        }

        let elements: Vec<Value> = response.json().await
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Unstructured API response: {}", e)))?;

        let mut documents = Vec::new();
        for element in elements {
            let text = element.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let el_type = element.get("type").and_then(|v| v.as_str()).unwrap_or("element");

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("element_type".to_string(), serde_json::Value::String(el_type.to_string()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("unstructured".to_string()));

            if let Some(metadata_obj) = element.get("metadata") {
                if let Some(page) = metadata_obj.get("page_number") {
                    metadata.insert("page_number".to_string(), page.clone());
                }
                if let Some(page) = metadata_obj.get("page_name") {
                    metadata.insert("page_name".to_string(), page.clone());
                }
            }

            documents.push(Document::new(text.to_string()).with_metadata(metadata));
        }

        Ok(documents)
    }
}

#[async_trait]
impl BaseLoader for UnstructuredLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        self.process().await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
