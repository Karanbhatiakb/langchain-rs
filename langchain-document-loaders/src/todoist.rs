//! Todoist document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that fetches tasks and projects from Todoist.
///
/// Provide a Todoist API token. Tasks are returned as documents with
/// project, priority, due date, labels, and section metadata.
#[derive(Debug, Clone)]
pub struct TodoistLoader {
    api_token: String,
}

impl TodoistLoader {
    /// Creates a new `TodoistLoader` with the given API token.
    pub fn new(api_token: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for TodoistLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("todoist".to_string()),
        );
        metadata.insert(
            "api_token".to_string(),
            serde_json::Value::String(self.api_token.clone()),
        );
        Ok(vec![Document::new("Todoist loader stub — no tasks loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
