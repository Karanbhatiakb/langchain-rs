//! Slack document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct SlackLoader {
    token: String,
    channel_id: Option<String>,
    client: Client,
}

impl SlackLoader {
    pub fn new() -> Self {
        let token = std::env::var("SLACK_BOT_TOKEN")
            .expect("SLACK_BOT_TOKEN environment variable is required");
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
        let url = format!("https://slack.com/api{}", endpoint);
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Slack API request failed: {}", e)))?;

        let body = response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Slack API response: {}", e)))?;

        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Slack API response: {}", e)))?;

        if value.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            let error = value.get("error").and_then(|v| v.as_str()).unwrap_or("unknown_error");
            return Err(ChainError::IOError(format!("Slack API error: {}", error)));
        }

        Ok(body)
    }

    pub async fn list_channels(&self) -> Result<Vec<Document>> {
        let body = self.api_get("/conversations.list?types=public_channel,private_channel").await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse channels: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(channels) = value.get("channels").and_then(|c| c.as_array()) {
            for channel in channels {
                let name = channel.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                let id = channel.get("id").and_then(|v| v.as_str()).unwrap_or("");

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(format!("slack:#{}", name)));
                metadata.insert("channel_id".to_string(), serde_json::Value::String(id.to_string()));
                metadata.insert("channel_name".to_string(), serde_json::Value::String(name.to_string()));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("slack".to_string()));

                documents.push(Document::new(serde_json::to_string(channel).unwrap_or_default()).with_metadata(metadata));
            }
        }

        Ok(documents)
    }

    pub async fn get_channel_messages(&self, channel_id: Option<&str>) -> Result<Vec<Document>> {
        let ch_id = channel_id.or(self.channel_id.as_deref())
            .ok_or_else(|| ChainError::ValidationError("No channel_id provided".to_string()))?;

        let body = self.api_get(&format!("/conversations.history?channel={}&limit=100", ch_id)).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse messages: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(messages) = value.get("messages").and_then(|m| m.as_array()) {
            for msg in messages {
                let text = msg.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let user = msg.get("user").and_then(|v| v.as_str()).unwrap_or("unknown");
                let ts = msg.get("ts").and_then(|v| v.as_str()).unwrap_or("");

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), serde_json::Value::String(format!("slack:{}", ch_id)));
                metadata.insert("channel_id".to_string(), serde_json::Value::String(ch_id.to_string()));
                metadata.insert("user".to_string(), serde_json::Value::String(user.to_string()));
                metadata.insert("timestamp".to_string(), serde_json::Value::String(ts.to_string()));
                metadata.insert("loader_type".to_string(), serde_json::Value::String("slack".to_string()));

                documents.push(Document::new(text.to_string()).with_metadata(metadata));
            }
        }

        Ok(documents)
    }
}

impl Default for SlackLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for SlackLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        if let Some(ch_id) = &self.channel_id {
            self.get_channel_messages(Some(ch_id)).await
        } else {
            self.list_channels().await
        }
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
