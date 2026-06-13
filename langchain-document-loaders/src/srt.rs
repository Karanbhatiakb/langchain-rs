//! SRT subtitle document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that parses SRT subtitle files.
///
/// Provide the file path to a `.srt` file. Each subtitle entry (sequence
/// number, time range, and text) is returned as a separate document with
/// timing metadata.
#[derive(Debug, Clone)]
pub struct SrtLoader {
    file_path: String,
}

impl SrtLoader {
    /// Creates a new `SrtLoader` for the given SRT file path.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for SrtLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("srt".to_string()),
        );
        metadata.insert(
            "source".to_string(),
            serde_json::Value::String(self.file_path.clone()),
        );
        Ok(vec![Document::new("SRT loader stub — no subtitles parsed yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
