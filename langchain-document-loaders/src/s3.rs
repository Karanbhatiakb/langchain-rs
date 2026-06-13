//! Amazon S3 document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

use crate::traits::BaseLoader;

#[allow(dead_code)]
pub struct S3Loader {
    bucket: String,
    region: String,
    key: String,
    access_key: String,
    secret_key: String,
    client: Client,
}

impl S3Loader {
    pub fn new(
        bucket: impl Into<String>,
        region: impl Into<String>,
        key: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        Self {
            bucket: bucket.into(),
            region: region.into(),
            key: key.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            client: Client::new(),
        }
    }

    fn build_url(&self) -> String {
        format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket, self.region, self.key
        )
    }
}

#[async_trait]
impl BaseLoader for S3Loader {
    async fn load(&self) -> Result<Vec<Document>> {
        let response = self
            .client
            .get(self.build_url())
            .header("Content-Type", "application/octet-stream")
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("S3 request failed for '{}': {}", self.key, e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::IOError(format!(
                "S3 error ({}): {}",
                status, body
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read S3 response: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(format!("s3://{}/{}", self.bucket, self.key)));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("s3".to_string()));
        metadata.insert("bucket".to_string(), serde_json::Value::String(self.bucket.clone()));
        metadata.insert("key".to_string(), serde_json::Value::String(self.key.clone()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
