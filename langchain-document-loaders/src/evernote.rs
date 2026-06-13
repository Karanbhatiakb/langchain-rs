//! Evernote document loader.
//!
//! Reads an Evernote export file (`.enex` — XML format) and returns each
//! note as a separate document.  Basic XML splitting is used; for production
//! use a proper XML parser with ENEX schema support.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

/// Loads notes from an Evernote ENEX export file.
#[derive(Debug, Clone)]
pub struct EvernoteLoader {
    file_path: String,
}

impl EvernoteLoader {
    /// Create a new `EvernoteLoader`.
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for EvernoteLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read ENEX file '{}': {}", self.file_path, e)))?;

        let mut documents = Vec::new();
        let mut note_start = 0;
        while let Some(start) = content[note_start..].find("<note>") {
            let abs_start = note_start + start;
            if let Some(end) = content[abs_start..].find("</note>") {
                let abs_end = abs_start + end + 7;
                let note_xml = &content[abs_start..abs_end];
                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("evernote".to_string()));
                documents.push(Document::new(note_xml.to_string()).with_metadata(metadata));
                note_start = abs_end;
            } else {
                break;
            }
        }

        if documents.is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("evernote".to_string()));
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
