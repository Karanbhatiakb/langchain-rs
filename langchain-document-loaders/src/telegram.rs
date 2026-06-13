//! Telegram document loader.
//!
//! Fetches messages from a Telegram chat using the Bot API.
//! Stub implementation — configure with your bot token and chat ID.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

/// Loads messages from a Telegram chat via the Bot API.
#[derive(Debug, Clone)]
pub struct TelegramLoader {
    bot_token: String,
    chat_id: String,
    client: Client,
}

impl TelegramLoader {
    /// Create a new `TelegramLoader`.
    pub fn new(bot_token: impl Into<String>, chat_id: impl Into<String>) -> Self {
        Self {
            bot_token: bot_token.into(),
            chat_id: chat_id.into(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl BaseLoader for TelegramLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates",
            self.bot_token
        );

        let payload = serde_json::json!({
            "chat_id": self.chat_id
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Telegram API request failed: {}", e)))?;

        let body = response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Telegram response: {}", e)))?;

        let mut metadata = HashMap::new();
        metadata.insert("chat_id".to_string(), serde_json::Value::String(self.chat_id.clone()));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("telegram".to_string()));

        Ok(vec![Document::new(body).with_metadata(metadata)])
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
