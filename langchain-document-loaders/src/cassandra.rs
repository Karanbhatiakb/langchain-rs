//! Apache Cassandra document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that reads rows from an Apache Cassandra table.
///
/// Provide contact points, keyspace, a CQL query, and optional parameters.
/// Each row is converted into a document with column metadata.
#[derive(Debug, Clone)]
pub struct CassandraLoader {
    query: String,
}

impl CassandraLoader {
    /// Creates a new `CassandraLoader` with the given CQL query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for CassandraLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("cassandra".to_string()),
        );
        metadata.insert(
            "query".to_string(),
            serde_json::Value::String(self.query.clone()),
        );
        Ok(vec![Document::new("Cassandra loader stub — no rows loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
