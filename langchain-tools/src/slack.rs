//! Slack messaging tool.

use async_trait::async_trait;
use serde_json::{json, Value};

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct SlackTool {
    token: String,
    client: reqwest::Client,
}

impl SlackTool {
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
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers
    }
}

#[async_trait]
impl BaseTool for SlackTool {
    fn name(&self) -> &str {
        "slack"
    }

    fn description(&self) -> &str {
        "Slack API tool. Supports: send_message <channel>\\n<message>, list_channels, get_channel_history <channel_id>\\n<limit>, search_messages <query>. Requires SLACK_BOT_TOKEN env var."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if input == "list_channels" {
            let url = "https://slack.com/api/conversations.list";
            let resp = self
                .client
                .get(url)
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Slack API error: {}", e)))?;

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

            if !result["ok"].as_bool().unwrap_or(false) {
                return Err(ChainError::ToolError(format!("Slack error: {}", result["error"].as_str().unwrap_or(""))));
            }

            let channels = result["channels"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for ch in channels {
                output.push(format!(
                    "#{} ({}) - {}",
                    ch["name"].as_str().unwrap_or(""),
                    ch["id"].as_str().unwrap_or(""),
                    if ch["is_member"].as_bool().unwrap_or(false) { "member" } else { "" },
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("send_message ") {
            let parts: Vec<&str> = cmd.splitn(2, '\n').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError("Usage: send_message <channel>\\n<message>".into()));
            }
            let channel = parts[0].trim();
            let message = parts[1].trim();

            let url = "https://slack.com/api/chat.postMessage";
            let resp = self
                .client
                .post(url)
                .headers(self.headers())
                .json(&json!({"channel": channel, "text": message}))
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Slack API error: {}", e)))?;

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

            if !result["ok"].as_bool().unwrap_or(false) {
                return Err(ChainError::ToolError(format!("Slack error: {}", result["error"].as_str().unwrap_or(""))));
            }

            return Ok("Message sent successfully".to_string());
        }

        if let Some(cmd) = input.strip_prefix("get_channel_history ") {
            let parts: Vec<&str> = cmd.splitn(2, '\n').collect();
            let channel = parts[0].trim();
            let limit = if parts.len() > 1 {
                parts[1].trim().parse::<u32>().unwrap_or(10)
            } else {
                10
            };

            let url = format!("https://slack.com/api/conversations.history?channel={}&limit={}", channel, limit);
            let resp = self
                .client
                .get(&url)
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Slack API error: {}", e)))?;

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

            if !result["ok"].as_bool().unwrap_or(false) {
                return Err(ChainError::ToolError(format!("Slack error: {}", result["error"].as_str().unwrap_or(""))));
            }

            let messages = result["messages"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for msg in messages {
                output.push(format!(
                    "<{}> {}",
                    msg["user"].as_str().unwrap_or("unknown"),
                    msg["text"].as_str().unwrap_or(""),
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("search_messages ") {
            let query = cmd.trim();
            let url = format!("https://slack.com/api/search.messages?query={}", urlencode(query));
            let resp = self
                .client
                .get(&url)
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Slack API error: {}", e)))?;

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

            if !result["ok"].as_bool().unwrap_or(false) {
                return Err(ChainError::ToolError(format!("Slack error: {}", result["error"].as_str().unwrap_or(""))));
            }

            let messages = result["messages"]["matches"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for msg in messages {
                output.push(format!(
                    "[{}] <{}>: {}",
                    msg["channel"]["name"].as_str().unwrap_or(""),
                    msg["username"].as_str().unwrap_or(""),
                    msg["text"].as_str().unwrap_or(""),
                ));
            }
            return Ok(output.join("\n"));
        }

        Err(ChainError::ToolError(
            "Unknown Slack command. Supported: send_message, list_channels, get_channel_history, search_messages".into(),
        ))
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => '+'.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
