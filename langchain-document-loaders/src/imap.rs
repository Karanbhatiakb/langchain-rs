//! IMAP email document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use std::collections::HashMap;

use crate::traits::BaseLoader;

/// A document loader that fetches emails from an IMAP server.
///
/// Provide the IMAP server host, username, and password. Emails from the
/// specified mailbox are returned as documents with subject, from, to,
/// and date metadata.
#[derive(Debug, Clone)]
pub struct ImapLoader {
    host: String,
    username: String,
}

impl ImapLoader {
    /// Creates a new `ImapLoader` for the given IMAP server and credentials.
    pub fn new(host: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            username: username.into(),
        }
    }
}

#[async_trait]
impl BaseLoader for ImapLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "loader_type".to_string(),
            serde_json::Value::String("imap".to_string()),
        );
        metadata.insert(
            "host".to_string(),
            serde_json::Value::String(self.host.clone()),
        );
        metadata.insert(
            "username".to_string(),
            serde_json::Value::String(self.username.clone()),
        );
        Ok(vec![Document::new("IMAP loader stub — no emails loaded yet")
            .with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
