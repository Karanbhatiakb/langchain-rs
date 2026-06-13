//! JSON document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::traits::BaseLoader;

pub struct JSONLoader {
    file_path: String,
    jq_schema: Option<String>,
}

impl JSONLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            jq_schema: None,
        }
    }

    pub fn with_jq_schema(mut self, schema: impl Into<String>) -> Self {
        self.jq_schema = Some(schema.into());
        self
    }
}

#[allow(dead_code)]
fn extract_fields(value: &Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                map.insert(k.clone(), v.clone());
            }
        }
        other => {
            map.insert("value".to_string(), other.clone());
        }
    }
    map
}

fn apply_schema(value: &Value, schema: &str) -> Result<Vec<Value>> {
    let schema = schema.trim();
    if schema == "." {
        return Ok(vec![value.clone()]);
    }
    if schema.starts_with(".") {
        let field = &schema[1..];
        match value {
            Value::Object(map) => {
                if let Some(v) = map.get(field) {
                    if v.is_array() {
                        return Ok(v.as_array().unwrap().clone());
                    }
                    return Ok(vec![v.clone()]);
                }
            }
            _ => {}
        }
    }
    if let Value::Array(arr) = value {
        return Ok(arr.clone());
    }
    Ok(vec![value.clone()])
}

#[async_trait]
impl BaseLoader for JSONLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to read JSON file '{}': {}", self.file_path, e)))?;

        let value: Value = serde_json::from_str(&content)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse JSON: {}", e)))?;

        let items = if let Some(schema) = &self.jq_schema {
            apply_schema(&value, schema)?
        } else {
            match &value {
                Value::Array(arr) => arr.clone(),
                _ => vec![value.clone()],
            }
        };

        let mut documents = Vec::new();
        for item in items {
            let content_str = serde_json::to_string(&item)
                .unwrap_or_else(|_| "{}".to_string());

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("json".to_string()));

            documents.push(Document::new(content_str).with_metadata(metadata));
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

pub struct JSONLinesLoader {
    file_path: String,
}

impl JSONLinesLoader {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for JSONLinesLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let file = fs::File::open(&self.file_path)
            .await
            .map_err(|e| ChainError::IOError(format!("Failed to open JSONL file '{}': {}", self.file_path, e)))?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut documents = Vec::new();
        let mut line_num = 0u64;
        while let Some(line) = lines.next_line().await
            .map_err(|e| ChainError::IOError(format!("Failed to read JSONL line: {}", e)))?
        {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(&trimmed)
                .map_err(|e| ChainError::ParserError(format!("Failed to parse JSONL line {}: {}", line_num, e)))?;

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(self.file_path.clone()));
            metadata.insert("line".to_string(), serde_json::Value::Number(line_num.into()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("jsonl".to_string()));

            documents.push(Document::new(
                serde_json::to_string(&value).unwrap_or_else(|_| trimmed)
            ).with_metadata(metadata));

            line_num += 1;
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
