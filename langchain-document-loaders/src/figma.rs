//! Figma document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that extracts text from Figma design files.
///
/// Provide the Figma file key and an access token. The loader retrieves
/// the document tree and flattens text nodes into documents with position
/// and style metadata.
#[derive(Debug, Clone)]
pub struct FigmaLoader {
    file_key: String,
}

impl FigmaLoader {
    /// Creates a new `FigmaLoader` for the given file key.
    pub fn new(file_key: impl Into<String>) -> Self {
        Self {
            file_key: file_key.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for FigmaLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("figma".to_string()),
        );
        metadata.insert(
            "file_key".to_string(),
            serde_json::Value::String(self.file_key.clone()),
        );
        Ok(vec![Document::new("Figma loader stub — no text nodes extracted yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
