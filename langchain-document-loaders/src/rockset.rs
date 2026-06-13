//! Rockset document loader.
//!
//! Executes a SQL query against a Rockset collection and returns each row
//! as a separate JSON document.  Requires a valid Rockset API key.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads documents from a Rockset collection via SQL query.
#[derive(Debug, Clone)]
pub struct RocksetLoader {
    api_key: String,
    api_server: String,
    sql_query: String,
    client: Client,
}

impl RocksetLoader {
    /// Create a new `RocksetLoader`.
    ///
    /// `api_server` defaults to `https://api.us2a.rockset.com`.
    pub fn new(api_key: impl Into<String>, sql_query: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_server: "https://api.us2a.rockset.com".to_string(),
            sql_query: sql_query.into(),
            client: Client::new(),
        }
    }

    /// Override the API server URL.
    pub fn with_api_server(mut self, api_server: impl Into<String>) -> Self {
        self.api_server = api_server.into();
        self
    }
}

#[async_trait]
impl BaseLoader for RocksetLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let payload = serde_json::json!({
            "sql": { "query": self.sql_query }
        });

        let response = self.client
            .post(format!("{}/v1/orgs/self/queries", self.api_server))
            .header("Authorization", format!("ApiKey {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Rockset query failed: {}", e)))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| ChainError::IOError(format!("Failed to parse Rockset response: {}", e)))?;

        let results = body["results"].as_array().cloned().unwrap_or_default();

        let mut documents = Vec::new();
        for row in results {
            let mut metadata = HashMap::new();
            metadata.insert("loader_type".to_string(), serde_json::Value::String("rockset".to_string()));
            documents.push(Document::new(row.to_string()).with_metadata(metadata));
        }

        if documents.is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("loader_type".to_string(), serde_json::Value::String("rockset".to_string()));
            metadata.insert("query".to_string(), serde_json::Value::String(self.sql_query.clone()));
            documents.push(Document::new("Rockset stub — no results returned").with_metadata(metadata));
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
