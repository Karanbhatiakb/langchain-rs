//! Email (`.eml`) document loader.
//!
//! Reads an email message in RFC 5322 / MIME format and returns the raw
//! content as a single document.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

/// Loads an `.eml` email file.
#[derive(Debug, Clone)]
pub struct EmailLoader {
    file_path: String,
}

impl EmailLoader {
    /// Create a new `EmailLoader`.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for EmailLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read email file '{}': {}", self.file_path, e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("email".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
