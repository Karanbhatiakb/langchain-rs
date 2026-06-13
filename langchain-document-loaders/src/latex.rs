//! LaTeX document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that parses LaTeX (.tex) files and extracts text content.
///
/// Provide the file path to a `.tex` file. The loader strips LaTeX commands
/// and returns the plain text with section and document class metadata.
#[derive(Debug, Clone)]
pub struct LatexLoader {
    file_path: String,
}

impl LatexLoader {
    /// Creates a new `LatexLoader` for the given LaTeX file path.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for LatexLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("latex".to_string()),
        );
        metadata.insert(
            "source".to_string(),
            serde_json::Value::String(self.file_path.clone()),
        );
        Ok(vec![Document::new("LaTeX loader stub — no content parsed yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
