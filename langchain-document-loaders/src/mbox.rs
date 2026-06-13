//! Mbox email archive document loader.
//!
//! Reads an mbox file (Unix mailbox format) containing multiple emails and
//! returns each message as a separate document.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

/// Loads emails from an mbox archive file.
#[derive(Debug, Clone)]
pub struct MboxLoader {
    file_path: String,
}

impl MboxLoader {
    /// Create a new `MboxLoader`.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for MboxLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read mbox file '{}': {}", self.file_path, e)))?;

        let mut documents = Vec::new();
        let mut offset = 0;

        while let Some(pos) = content[offset..].find("\nFrom ") {
            let abs_pos = offset + pos;
            if abs_pos > 0 {
                let msg = &content[offset..abs_pos].trim().to_string();
                if !msg.is_empty() {
                    let mut metadata = HashMap::new();
                    metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
                    metadata.insert("loader_type".to_string(), serde_json::Value::String("mbox".to_string()));
                    documents.push(Document::new(msg.clone()).with_metadata(metadata));
                }
            }
            offset = abs_pos + 1;
        }

        let remaining = content[offset..].trim().to_string();
        if !remaining.is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("mbox".to_string()));
            documents.push(Document::new(remaining).with_metadata(metadata));
        }

        if documents.is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("mbox".to_string()));
            documents.push(Document::new(content).with_metadata(metadata));
        }

        Ok(documents)
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
