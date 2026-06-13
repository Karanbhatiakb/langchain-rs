//! Databricks document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that reads data from a Databricks workspace.
///
/// Supports querying Unity Catalog tables or running SQL queries via the
/// Databricks SQL warehouse or Spark SQL endpoint.
#[derive(Debug, Clone)]
pub struct DatabricksLoader {
    query: String,
}

impl DatabricksLoader {
    /// Creates a new `DatabricksLoader` with the given query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for DatabricksLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("databricks".to_string()),
        );
        metadata.insert(
            "query".to_string(),
            serde_json::Value::String(self.query.clone()),
        );
        Ok(vec![Document::new("Databricks loader stub — no data loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
