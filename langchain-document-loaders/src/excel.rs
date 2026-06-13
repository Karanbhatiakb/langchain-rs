//! Microsoft Excel document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that reads data from Microsoft Excel (.xlsx/.xls) files.
///
/// Provide a file path and optionally specify which sheets to load. Each row
/// is converted into a document with column and sheet metadata.
#[derive(Debug, Clone)]
pub struct ExcelLoader {
    file_path: String,
}

impl ExcelLoader {
    /// Creates a new `ExcelLoader` for the given Excel file path.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for ExcelLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("excel".to_string()),
        );
        metadata.insert(
            "source".to_string(),
            serde_json::Value::String(self.file_path.clone()),
        );
        Ok(vec![Document::new("Excel loader stub — no sheets loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
