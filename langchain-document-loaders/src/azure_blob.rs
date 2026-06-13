//! Azure Blob Storage document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

use crate::traits::BaseLoader;

pub struct AzureBlobLoader {
    account: String,
    container: String,
    blob: String,
    sas_token: String,
    client: Client,
}

impl AzureBlobLoader {
    pub fn new(
        account: impl Into<String>,
        container: impl Into<String>,
        blob: impl Into<String>,
        sas_token: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            container: container.into(),
            blob: blob.into(),
            sas_token: sas_token.into(),
            client: Client::new(),
        }
    }

    fn build_url(&self) -> String {
        format!(
            "https://{}.blob.core.windows.net/{}/{}?{}",
            self.account, self.container, self.blob, self.sas_token
        )
    }
}

#[async_trait]
impl BaseLoader for AzureBlobLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self
            .client
            .get(self.build_url())
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Azure Blob request failed for '{}': {}", self.blob, e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::IOError(format!(
                "Azure Blob error ({}): {}",
                status, body
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read Azure Blob response: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(format!("https://{}.blob.core.windows.net/{}/{}", self.account, self.container, self.blob)));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("azure_blob".to_string()));
        metadata.insert("account".to_string(), serde_json::Value::String(self.account.clone()));
        metadata.insert("container".to_string(), serde_json::Value::String(self.container.clone()));
        metadata.insert("blob".to_string(), serde_json::Value::String(self.blob.clone()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
