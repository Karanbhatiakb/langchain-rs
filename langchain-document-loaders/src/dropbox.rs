//! Dropbox document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::traits::BaseLoader;

const DROPBOX_API_URL: &str = "https://api.dropboxapi.com/2/files/download";

pub struct DropboxLoader {
    access_token: String,
    file_path: String,
    client: Client,
}

impl DropboxLoader {
    pub fn new(access_token: impl Into<String>, file_path: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            file_path: file_path.into(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct DropboxDownloadArg {
    path: String,
}

#[async_trait]
impl BaseLoader for DropboxLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let dropbox_api_arg = serde_json::to_string(&DropboxDownloadArg {
            path: self.file_path.clone(),
        })
        .map_err(|e| ChainError::SerializationError(format!("Failed to serialize Dropbox arg: {}", e)))?;

        let response = self
            .client
            .post(DROPBOX_API_URL)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Dropbox-API-Arg", &dropbox_api_arg)
            .header("Content-Type", "application/octet-stream")
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Dropbox request failed for '{}': {}", self.file_path, e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::IOError(format!(
                "Dropbox error ({}): {}",
                status, body
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read Dropbox response: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("dropbox".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
