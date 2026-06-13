//! Iugu document loader.
//!
//! Fetches data from the Iugu payment / invoicing API.
//! Stub implementation — configure with your Iugu API token.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// Loads invoices and payment data from the Iugu API.
#[derive(Debug, Clone)]
pub struct IuguLoader {
    api_token: String,
    endpoint: String,
}

impl IuguLoader {
    /// Create a new `IuguLoader`.
    ///
    /// `endpoint` can be e.g. `"invoices"` or `"charges"`.
    pub fn new(api_token: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for IuguLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert("loader_type".to_string(), serde_json::Value::String("iugu".to_string()));
        metadata.insert("endpoint".to_string(), serde_json::Value::String(self.endpoint.clone()));

        Ok(vec![Document::new(format!(
            "IuguLoader stub — configured for endpoint '{}' with token '{}'",
            self.endpoint,
            self.api_token.chars().map(|c| if c.is_alphanumeric() { c } else { '*' }).collect::<String>()
        )).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
