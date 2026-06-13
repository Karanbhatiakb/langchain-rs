//! Spreedly document loader.
//!
//! Fetches payment / transaction data from the Spreedly API.
//! Stub implementation — configure with your Spreedly environment key
//! and access secret.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// Loads transaction data from the Spreedly API.
#[derive(Debug, Clone)]
pub struct SpreedlyLoader {
    environment_key: String,
    gateway_token: Option<String>,
}

impl SpreedlyLoader {
    /// Create a new `SpreedlyLoader`.
    pub fn new(environment_key: impl Into<String>, _access_secret: impl Into<String>) -> Self {
        Self {
            environment_key: environment_key.into(),
            gateway_token: None,
        }
    }

    /// Set the gateway token to scope transactions.
    pub fn with_gateway_token(mut self, token: impl Into<String>) -> Self {
        self.gateway_token = Some(token.into());
        self
    }
}

#[async_trait]
impl BaseLoader for SpreedlyLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert("loader_type".to_string(), serde_json::Value::String("spreedly".to_string()));
        metadata.insert("environment_key".to_string(), serde_json::Value::String(self.environment_key.clone()));

        let msg = match &self.gateway_token {
            Some(gateway) => format!("SpreedlyLoader stub — gateway '{}'", gateway),
            None => "SpreedlyLoader stub — all transactions".to_string(),
        };

        Ok(vec![Document::new(msg).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
