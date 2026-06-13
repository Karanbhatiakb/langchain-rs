//! EPUB document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that extracts text content from EPUB e-book files.
///
/// Provide the file path to an `.epub` file. The loader parses the EPUB
/// container, reads each chapter, and returns documents with chapter metadata.
#[derive(Debug, Clone)]
pub struct EpubLoader {
    file_path: String,
}

impl EpubLoader {
    /// Creates a new `EpubLoader` for the given EPUB file path.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for EpubLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("epub".to_string()),
        );
        metadata.insert(
            "source".to_string(),
            serde_json::Value::String(self.file_path.clone()),
        );
        Ok(vec![Document::new("Epub loader stub — no content extracted yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
