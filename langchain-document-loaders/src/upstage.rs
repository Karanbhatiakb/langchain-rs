//! Upstage document loader.

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

const UPSTAGE_BASE_URL: &str = "https://api.upstage.ai/v1/document-ai";

pub struct UpstageLoader {
    api_key: String,
    document: String,
    output_format: String,
    client: Client,
}

impl UpstageLoader {
    pub fn new(api_key: impl Into<String>, document: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            document: document.into(),
            output_format: "text".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_output_format(mut self, format: impl Into<String>) -> Self {
        self.output_format = format.into();
        self
    }
}

#[derive(Deserialize)]
struct UpstageResponse {
    elements: Vec<UpstageElement>,
}

#[derive(Deserialize)]
struct UpstageElement {
    content: String,
}

#[async_trait]
impl BaseLoader for UpstageLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self
            .client
            .post(format!("{}/document-parse", UPSTAGE_BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "document": self.document,
                "output_format": self.output_format,
            }))
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Upstage request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::IOError(format!(
                "Upstage error ({}): {}",
                status, body
            )));
        }

        let result: UpstageResponse = response.json().await.map_err(|e| {
            ChainError::ParserError(format!("Failed to parse Upstage response: {}", e))
        })?;

        let documents: Vec<Document> = result
            .elements
            .into_iter()
            .map(|el| {
                let mut metadata = HashMap::new();
                metadata.insert("loader_type".to_string(), serde_json::Value::String("upstage".to_string()));
                metadata.insert("source".to_string(), serde_json::Value::String(self.document.clone()));
                Document::new(el.content).with_metadata(metadata)
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
