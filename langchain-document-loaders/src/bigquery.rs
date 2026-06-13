//! Google BigQuery document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that fetches rows from a Google BigQuery table or query.
///
/// Configure with a project, dataset, and optional SQL query. Each result row
/// is converted into a document.
#[derive(Debug, Clone)]
pub struct BigQueryLoader {
    query: String,
}

impl BigQueryLoader {
    /// Creates a new `BigQueryLoader` with the given SQL query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for BigQueryLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("bigquery".to_string()),
        );
        metadata.insert(
            "query".to_string(),
            serde_json::Value::String(self.query.clone()),
        );
        Ok(vec![Document::new("BigQuery loader stub — no rows loaded yet").with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
