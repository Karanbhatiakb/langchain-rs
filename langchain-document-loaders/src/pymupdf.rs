//! PyMuPDF document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;

use crate::traits::BaseLoader;

pub struct PyMuPDFLoader {
    file_path: String,
}

impl PyMuPDFLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for PyMuPDFLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = tokio::fs::read(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read PDF '{}': {}", self.file_path, e)))?;

        let text = extract_text_from_pdf(&content)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse PDF '{}': {}", self.file_path, e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("pymupdf".to_string()));

        Ok(vec![Document::new(text).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}

fn extract_text_from_pdf(data: &[u8]) -> std::result::Result<String, String> {
    let mut text = String::new();
    let mut pos = 0;
    let len = data.len();

    while pos < len {
        if let Some(start) = find_pattern(data, b"BT", pos) {
            if let Some(end) = find_pattern(data, b"ET", start + 2) {
                let segment = &data[start + 2..end];
                text.push_str(&extract_text_from_segment(segment));
                pos = end + 2;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(text)
}

fn find_pattern(data: &[u8], pattern: &[u8], start: usize) -> Option<usize> {
    if start >= data.len() {
        return None;
    }
    data[start..]
        .windows(pattern.len())
        .position(|w| w == pattern)
        .map(|i| i + start)
}

fn extract_text_from_segment(segment: &[u8]) -> String {
    let mut text = String::new();
    let mut i = 0;

    while i < segment.len() {
        if segment[i] == b'(' {
            let mut depth = 1;
            i += 1;
            while i < segment.len() && depth > 0 {
                match segment[i] {
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    b'\\' if i + 1 < segment.len() => i += 1,
                    _ => {}
                }
                if depth > 0 {
                    text.push(segment[i] as char);
                }
                i += 1;
            }
            text.push(' ');
        } else {
            i += 1;
        }
    }

    text.trim().to_string()
}
