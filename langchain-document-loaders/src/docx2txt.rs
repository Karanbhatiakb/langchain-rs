//! Docx2txt document loader — extracts plain text from `.docx` files.
//!
//! This loader reads a Word document at the given path and returns its
//! raw XML content as a single document.  For production use, pair this
//! with a proper docx-to-text converter.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

/// Loader that reads a `.docx` file and returns its content as text.
#[derive(Debug, Clone)]
pub struct Docx2txtLoader {
    file_path: String,
}

impl Docx2txtLoader {
    /// Create a new `Docx2txtLoader` for the given file path.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for Docx2txtLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read docx file '{}': {}", self.file_path, e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("docx2txt".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
