//! Audio document loader (transcription).

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

pub struct AudioLoader {
    file_path: String,
}

impl AudioLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for AudioLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let file_metadata = fs::metadata(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read audio file '{}': {}", self.file_path, e)))?;

        let path = std::path::Path::new(&self.file_path);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let modified = file_metadata.modified().ok().map(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "unknown".to_string())
        }).unwrap_or_else(|| "unknown".to_string());

        let content = format!(
            "Audio file: {}\nFormat: {}\nSize: {} bytes\nModified: {}",
            file_name,
            ext.to_uppercase(),
            file_metadata.len(),
            modified,
        );

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("file_name".to_string(), serde_json::Value::String(file_name));
        metadata.insert("format".to_string(), serde_json::Value::String(ext));
        metadata.insert("size".to_string(), serde_json::Value::Number(file_metadata.len().into()));
        metadata.insert("modified".to_string(), serde_json::Value::String(modified));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("audio".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
