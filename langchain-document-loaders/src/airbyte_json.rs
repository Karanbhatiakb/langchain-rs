//! Airbyte JSON document loader.
//!
//! Reads an Airbyte JSON export file (newline-delimited JSON records) and
//! returns each record as a separate document.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

/// Loads documents from an Airbyte JSON export file.
#[derive(Debug, Clone)]
pub struct AirbyteJSONLoader {
    file_path: String,
}

impl AirbyteJSONLoader {
    /// Create a new `AirbyteJSONLoader`.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for AirbyteJSONLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read Airbyte JSON file '{}': {}", self.file_path, e)))?;

        let mut documents = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("airbyte_json".to_string()));
            documents.push(Document::new(trimmed.to_string()).with_metadata(metadata));
        }

        if documents.is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("airbyte_json".to_string()));
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
