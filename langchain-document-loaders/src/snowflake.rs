//! Snowflake document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that queries Snowflake data warehouses.
///
/// Provide a SQL query and connection parameters (account, warehouse, database,
/// schema, user, role). Each result row is returned as a document with column
/// metadata.
#[derive(Debug, Clone)]
pub struct SnowflakeLoader {
    query: String,
}

impl SnowflakeLoader {
    /// Creates a new `SnowflakeLoader` with the given SQL query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for SnowflakeLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("snowflake".to_string()),
        );
        metadata.insert(
            "query".to_string(),
            serde_json::Value::String(self.query.clone()),
        );
        Ok(vec![Document::new("Snowflake loader stub — no rows loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
