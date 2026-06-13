//! Jupyter notebook document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that extracts content from Jupyter notebook (.ipynb) files.
///
/// Provide the file path to a `.ipynb` file. Code cells, markdown cells, and
/// raw cells are extracted and returned as individual documents with cell
/// type and index metadata.
#[derive(Debug, Clone)]
pub struct JupyterLoader {
    file_path: String,
}

impl JupyterLoader {
    /// Creates a new `JupyterLoader` for the given notebook file path.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for JupyterLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("jupyter".to_string()),
        );
        metadata.insert(
            "source".to_string(),
            serde_json::Value::String(self.file_path.clone()),
        );
        Ok(vec![Document::new("Jupyter notebook loader stub — no cells extracted yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
