//! Discord messaging tool.

use async_trait::async_trait;
use serde_json::{json, Value};

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct DiscordTool {
    token: String,
    client: reqwest::Client,
}

impl DiscordTool {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bot {}", self.token)).unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }
}

#[async_trait]
impl BaseTool for DiscordTool {
    fn name(&self) -> &str {
        "discord"
    }

    fn description(&self) -> &str {
        "Discord API tool. Supports: send_message <channel_id>\\n<message>, get_channel_messages <channel_id>\\n<limit>. Requires DISCORD_BOT_TOKEN env var."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if let Some(cmd) = input.strip_prefix("send_message ") {
            let parts: Vec<&str> = cmd.splitn(2, '\n').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError("Usage: send_message <channel_id>\\n<message>".into()));
            }
            let channel_id = parts[0].trim();
            let message = parts[1].trim();

            let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
            let resp = self
                .client
                .post(&url)
                .headers(self.headers())
                .json(&json!({"content": message}))
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Discord API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Discord API returned {}", resp.status())));
            }

            return Ok("Message sent successfully".to_string());
        }

        if let Some(cmd) = input.strip_prefix("get_channel_messages ") {
            let parts: Vec<&str> = cmd.splitn(2, '\n').collect();
            let channel_id = parts[0].trim();
            let limit = if parts.len() > 1 {
                parts[1].trim().parse::<u32>().unwrap_or(10)
            } else {
                10
            };

            let url = format!(
                "https://discord.com/api/v10/channels/{}/messages?limit={}",
                channel_id, limit
            );
            let resp = self
                .client
                .get(&url)
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Discord API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Discord API returned {}", resp.status())));
            }

            let messages: Vec<Value> = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let mut output = Vec::new();
            for msg in messages {
                output.push(format!(
                    "<{}> {}",
                    msg["author"]["username"].as_str().unwrap_or("unknown"),
                    msg["content"].as_str().unwrap_or(""),
                ));
            }
            return Ok(output.join("\n"));
        }

        Err(ChainError::ToolError(
            "Unknown Discord command. Supported: send_message, get_channel_messages".into(),
        ))
    }
}
