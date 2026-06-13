//! Text file document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

pub struct TextLoader {
    file_path: String,
    encoding: String,
}

impl TextLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            encoding: "utf-8".to_string(),
        }
    }

    pub fn with_encoding(mut self, encoding: impl Into<String>) -> Self {
        self.encoding = encoding.into();
        self
    }
}

#[async_trait]
impl BaseLoader for TextLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read file '{}': {}", self.file_path, e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("text".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
