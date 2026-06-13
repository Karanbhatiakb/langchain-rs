//! Box document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

use crate::traits::BaseLoader;

const BOX_API_BASE: &str = "https://api.box.com/2.0";

pub struct BoxLoader {
    access_token: String,
    file_id: String,
    client: Client,
}

impl BoxLoader {
    pub fn new(access_token: impl Into<String>, file_id: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            file_id: file_id.into(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl BaseLoader for BoxLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let url = format!("{}/files/{}/content", BOX_API_BASE, self.file_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Box request failed for file '{}': {}", self.file_id, e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::IOError(format!(
                "Box error ({}): {}",
                status, body
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read Box response: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(url));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("box".to_string()));
        metadata.insert("file_id".to_string(), serde_json::Value::String(self.file_id.clone()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
