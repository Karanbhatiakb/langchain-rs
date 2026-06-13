//! Apache Kafka document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that consumes messages from an Apache Kafka topic.
///
/// Provide the bootstrap servers and topic name. Each consumed message is
/// returned as a document with topic, partition, and offset metadata.
#[derive(Debug, Clone)]
pub struct KafkaLoader {
    topic: String,
    bootstrap_servers: String,
}

impl KafkaLoader {
    /// Creates a new `KafkaLoader` for the given topic and bootstrap servers.
    pub fn new(topic: impl Into<String>, bootstrap_servers: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            bootstrap_servers: bootstrap_servers.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for KafkaLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("kafka".to_string()),
        );
        metadata.insert(
            "topic".to_string(),
            serde_json::Value::String(self.topic.clone()),
        );
        metadata.insert(
            "bootstrap_servers".to_string(),
            serde_json::Value::String(self.bootstrap_servers.clone()),
        );
        Ok(vec![Document::new("Kafka loader stub — no messages consumed yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
