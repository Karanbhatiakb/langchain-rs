//! MHTML document loader.
//!
//! Reads an `.mhtml` (MIME HTML archive) file and returns its raw content.
//! For production use, add a proper MHTML parser to extract individual
//! resources.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

/// Loads a saved MHTML (MIME HTML archive) file.
#[derive(Debug, Clone)]
pub struct MHTMLLoader {
    file_path: String,
}

impl MHTMLLoader {
    /// Create a new `MHTMLLoader`.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for MHTMLLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read MHTML file '{}': {}", self.file_path, e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("mhtml".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
