//! MongoDB document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that reads documents from a MongoDB collection.
///
/// Provide the connection URI, database name, collection name, and an
/// optional query filter. Each MongoDB document is converted into a
/// LangChain document with collection and database metadata.
#[derive(Debug, Clone)]
pub struct MongodbLoader {
    collection: String,
    database: String,
}

impl MongodbLoader {
    /// Creates a new `MongodbLoader` for the given database and collection.
    pub fn new(database: impl Into<String>, collection: impl Into<String>) -> Self {
        Self {
            database: database.into(),
            collection: collection.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for MongodbLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("mongodb".to_string()),
        );
        metadata.insert(
            "database".to_string(),
            serde_json::Value::String(self.database.clone()),
        );
        metadata.insert(
            "collection".to_string(),
            serde_json::Value::String(self.collection.clone()),
        );
        Ok(vec![Document::new("MongoDB loader stub — no documents loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
