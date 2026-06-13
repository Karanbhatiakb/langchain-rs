//! CSV document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;

use crate::traits::BaseLoader;

pub struct CSVLoader {
    file_path: String,
    delimiter: u8,
    quote_char: u8,
    has_header: bool,
}

impl CSVLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            delimiter: b',',
            quote_char: b'"',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_quote_char(mut self, quote_char: u8) -> Self {
        self.quote_char = quote_char;
        self
    }

    pub fn with_has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }
}

#[async_trait]
impl BaseLoader for CSVLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = tokio::fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read CSV '{}': {}", self.file_path, e)))?;

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .has_headers(self.has_header)
            .from_reader(content.as_bytes());

        let headers: Vec<String> = if self.has_header {
            reader.headers()
                .map_err(|e| ChainError::ParserError(format!("Failed to read CSV headers: {}", e)))?
                .iter()
                .map(|h| h.to_string())
                .collect()
        } else {
            (0..reader.headers()
                .map_err(|e| ChainError::ParserError(format!("Failed to read CSV record: {}", e)))?
                .len())
                .map(|i| format!("column_{}", i))
                .collect()
        };

        let mut documents = Vec::new();
        for (row_idx, result) in reader.records().enumerate() {
            let record = result
                .map_err(|e| ChainError::ParserError(format!("CSV parse error at row {}: {}", row_idx, e)))?;

            let mut fields = HashMap::new();
            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    fields.insert(headers[i].clone(), serde_json::Value::String(field.to_string()));
                }
            }

            let row_json = serde_json::to_string(&fields)
                .unwrap_or_else(|_| "{}".to_string());

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("row".to_string(), serde_json::Value::Number((row_idx as u64).into()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("csv".to_string()));

            documents.push(Document::new(row_json).with_metadata(metadata));
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
