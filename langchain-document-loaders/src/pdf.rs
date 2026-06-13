//! PDF document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use tokio::fs;

use crate::traits::BaseLoader;

pub struct PDFLoader {
    file_path: String,
}

impl PDFLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for PDFLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let bytes = fs::read(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read PDF file '{}': {}", self.file_path, e)))?;

        let content = extract_text_from_pdf(&bytes);

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("pdf".to_string()));
        metadata.insert("size".to_string(), serde_json::Value::Number(bytes.len().into()));

        Ok(vec![Document::new(content).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}

fn extract_text_from_pdf(data: &[u8]) -> String {
    let text = String::from_utf8_lossy(data);
    let mut result = String::new();
    let mut in_text = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("BT") {
            in_text = true;
            continue;
        }
        if trimmed.starts_with("ET") {
            in_text = false;
            continue;
        }
        if in_text {
            if let Some(t) = extract_pdf_text_operator(trimmed) {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(&t);
            }
        }
    }

    if result.is_empty() {
        let cleaned: Vec<&str> = text.lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty()
                    && !t.starts_with('%')
                    && !t.starts_with("<<")
                    && !t.starts_with(">>")
                    && !t.starts_with("obj")
                    && !t.starts_with("endobj")
                    && !t.starts_with("stream")
                    && !t.starts_with("endstream")
                    && !t.starts_with("xref")
                    && !t.starts_with("trailer")
                    && !t.starts_with("startxref")
                    && !t.contains("cm")
                    && !t.contains("Tm")
                    && !t.contains("Td")
                    && !t.starts_with('/')
                    && !t.starts_with('[')
                    && !t.starts_with(']')
                    && t.len() > 3
            })
            .collect();
        result = cleaned.join("\n");
    }

    result
}

fn extract_pdf_text_operator(line: &str) -> Option<String> {
    let line = line.trim();
    if line.ends_with("Tj") || line.ends_with("TJ") {
        let content = line.trim_end_matches("Tj").trim_end_matches("TJ").trim();
        let content = content.trim_matches('(').trim_matches(')');
        if !content.is_empty() && !content.contains('\\') {
            return Some(content.to_string());
        }
        let mut result = String::new();
        let mut chars = content.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        '(' => result.push('('),
                        ')' => result.push(')'),
                        '\\' => result.push('\\'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                }
            } else {
                result.push(c);
            }
        }
        if !result.is_empty() {
            return Some(result);
        }
    }
    None
}
