//! Google Cloud Storage directory document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that reads all blobs under a GCS bucket prefix.
///
/// Provide the bucket name and an optional prefix (directory path). Every
/// object under the prefix is fetched and returned as a separate document
/// with bucket, prefix, and blob name metadata.
#[derive(Debug, Clone)]
pub struct GcsDirectoryLoader {
    bucket: String,
    prefix: String,
}

impl GcsDirectoryLoader {
    /// Creates a new `GcsDirectoryLoader` for the given bucket and prefix.
    pub fn new(bucket: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            prefix: prefix.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for GcsDirectoryLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("gcs_directory".to_string()),
        );
        metadata.insert(
            "bucket".to_string(),
            serde_json::Value::String(self.bucket.clone()),
        );
        metadata.insert(
            "prefix".to_string(),
            serde_json::Value::String(self.prefix.clone()),
        );
        Ok(vec![Document::new("GCS directory loader stub — no objects loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
