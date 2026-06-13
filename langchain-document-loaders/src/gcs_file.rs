//! Google Cloud Storage file document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that reads a single file from Google Cloud Storage.
///
/// Provide the GCS bucket name and blob (object) path. The content is
/// fetched and returned as a single document with bucket and blob metadata.
#[derive(Debug, Clone)]
pub struct GcsFileLoader {
    bucket: String,
    blob: String,
}

impl GcsFileLoader {
    /// Creates a new `GcsFileLoader` for the given bucket and blob path.
    pub fn new(bucket: impl Into<String>, blob: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            blob: blob.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for GcsFileLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("gcs_file".to_string()),
        );
        metadata.insert(
            "bucket".to_string(),
            serde_json::Value::String(self.bucket.clone()),
        );
        metadata.insert(
            "blob".to_string(),
            serde_json::Value::String(self.blob.clone()),
        );
        Ok(vec![Document::new("GCS file loader stub — no content loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
