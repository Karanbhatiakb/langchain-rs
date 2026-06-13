//! Docling document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::traits::BaseLoader;

const DOCLING_DEFAULT_URL: &str = "http://localhost:5001";

pub struct DoclingLoader {
    endpoint: String,
    document: String,
    client: Client,
}

impl DoclingLoader {
    pub fn new(document: impl Into<String>) -> Self {
        Self {
            endpoint: DOCLING_DEFAULT_URL.to_string(),
            document: document.into(),
            client: Client::new(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }
}

#[derive(Deserialize)]
struct DoclingResponse {
    pages: Vec<DoclingPage>,
}

#[derive(Deserialize)]
struct DoclingPage {
    content: String,
    page_number: u32,
    #[serde(default)]
    metadata: HashMap<String, serde_json::Value>,
}

#[async_trait]
impl BaseLoader for DoclingLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self
            .client
            .post(format!("{}/process", self.endpoint))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "document": self.document,
            }))
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Docling request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::IOError(format!(
                "Docling error ({}): {}",
                status, body
            )));
        }

        let result: DoclingResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Docling response: {}", e))
        })?;

        let documents: Vec<Document> = result
            .pages
            .into_iter()
            .map(|page| {
                let mut metadata = HashMap::new();
                metadata.insert("loader_type".to_string(), serde_json::Value::String("docling".to_string()));
                metadata.insert("source".to_string(), serde_json::Value::String(self.document.clone()));
                metadata.insert("page_number".to_string(), serde_json::Value::Number(page.page_number.into()));
                metadata.extend(page.metadata);
                Document::new(page.content).with_metadata(metadata)
            })
            .collect();

        Ok(documents)
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
