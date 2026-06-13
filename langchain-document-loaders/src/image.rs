//! Image document loader (OCR).

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

pub struct ImageLoader {
    file_path: String,
}

impl ImageLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for ImageLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let metadata = fs::metadata(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read image file '{}': {}", self.file_path, e)))?;

        let path = std::path::Path::new(&self.file_path);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let content = format!(
            "Image file: {}\nType: {}\nSize: {} bytes\nModified: {:?}",
            file_name,
            ext.to_uppercase(),
            metadata.len(),
            metadata.modified().ok().map(|t| {
                let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| "unknown".to_string())
            }).unwrap_or_else(|| "unknown".to_string()),
        );

        let mut metadata_map = HashMap::new();
        metadata_map.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata_map.insert("file_name".to_string(), serde_json::Value::String(file_name));
        metadata_map.insert("extension".to_string(), serde_json::Value::String(ext));
        metadata_map.insert("size".to_string(), serde_json::Value::Number(metadata.len().into()));
        metadata_map.insert("loader_type".to_string(), serde_json::Value::String("image".to_string()));

        Ok(vec![Document::new(content).with_metadata(metadata_map)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
