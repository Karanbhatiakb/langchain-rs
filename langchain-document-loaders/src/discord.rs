//! Discord document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct DiscordLoader {
    token: String,
    channel_id: Option<String>,
    client: Client,
}

impl DiscordLoader {
    pub fn new() -> Self {
        let token = std::env::var("DISCORD_TOKEN")
            .expect("DISCORD_TOKEN environment variable is required");
        Self {
            token,
            channel_id: None,
            client: Client::new(),
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = token.into();
        self
    }

    pub fn with_channel(mut self, channel_id: impl Into<String>) -> Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    async fn api_get(&self, endpoint: &str) -> Result<String> {
        let url = format!("https://discord.com/api/v10{}", endpoint);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bot {}", self.token))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Discord API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Discord API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Discord API response: {}", e)))
    }

    pub async fn get_channel_messages(&self, channel_id: Option<&str>, limit: Option<u32>) -> Result<Vec<Document>> {
        let ch_id = channel_id.or(self.channel_id.as_deref())
            .ok_or_else(|| ChainError::ValidationError("No channel_id provided".to_string()))?;

        let limit = limit.unwrap_or(100);
        let body = self.api_get(&format!("/channels/{}/messages?limit={}", ch_id, limit)).await?;
        let messages: Vec<Value> = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Discord messages: {}", e)))?;

        let mut documents = Vec::new();
        for msg in &messages {
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let author_id = msg.get("author")
                .and_then(|a| a.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let author_name = msg.get("author")
                .and_then(|a| a.get("username"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let timestamp = msg.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String(
                format!("https://discord.com/channels/{}", ch_id)
            ));
            metadata.insert("channel_id".to_string(), serde_json::Value::String(ch_id.to_string()));
            metadata.insert("author_id".to_string(), serde_json::Value::String(author_id.to_string()));
            metadata.insert("author_name".to_string(), serde_json::Value::String(author_name.to_string()));
            metadata.insert("timestamp".to_string(), serde_json::Value::String(timestamp.to_string()));
            metadata.insert("loader_type".to_string(), serde_json::Value::String("discord".to_string()));

            documents.push(Document::new(content.to_string()).with_metadata(metadata));
        }

        Ok(documents)
    }
}

impl Default for DiscordLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for DiscordLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        self.get_channel_messages(None, None).await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
