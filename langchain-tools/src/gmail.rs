//! Gmail tool.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct GmailTool {
    api_key: String,
    client: reqwest::Client,
}

impl GmailTool {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for GmailTool {
    fn name(&self) -> &str {
        "gmail"
    }

    fn description(&self) -> &str {
        "Gmail API tool. Supports: list_emails <max_results>, read_email <message_id>, search_emails <query>, send_email <to>\\n<subject>\\n<body>. Requires GMAIL_API_KEY env var. Note: Full Gmail API requires OAuth 2.0 setup."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if let Some(cmd) = input.strip_prefix("list_emails ") {
            let max_results = cmd.trim().parse::<u32>().unwrap_or(10);
            let url = format!(
                "https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults={}&key={}",
                max_results, self.api_key
            );
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Gmail API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Gmail API returned {}", resp.status())));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let messages = result["messages"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for msg in messages {
                output.push(msg["id"].as_str().unwrap_or("").to_string());
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("read_email ") {
            let message_id = cmd.trim();
            let url = format!(
                "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?key={}",
                message_id, self.api_key
            );
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Gmail API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Gmail API returned {}", resp.status())));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let payload = &result["payload"];
            let mut subject = String::new();
            let mut from = String::new();
            if let Some(headers) = payload["headers"].as_array() {
                for h in headers {
                    match h["name"].as_str() {
                        Some("Subject") => subject = h["value"].as_str().unwrap_or("").to_string(),
                        Some("From") => from = h["value"].as_str().unwrap_or("").to_string(),
                        _ => {}
                    }
                }
            }

            return Ok(format!("From: {}\nSubject: {}\n\n(Full body requires OAuth 2.0)", from, subject));
        }

        if let Some(cmd) = input.strip_prefix("search_emails ") {
            let query = cmd.trim();
            let url = format!(
                "https://gmail.googleapis.com/gmail/v1/users/me/messages?q={}&key={}",
                urlencode(query), self.api_key
            );
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Gmail API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Gmail API returned {}", resp.status())));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let messages = result["messages"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for msg in messages {
                output.push(msg["id"].as_str().unwrap_or("").to_string());
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("send_email ") {
            let parts: Vec<&str> = cmd.splitn(3, '\n').collect();
            if parts.len() < 3 {
                return Err(ChainError::ToolError("Usage: send_email <to>\\n<subject>\\n<body>".into()));
            }
            return Err(ChainError::ToolError(
                "Gmail send requires OAuth 2.0 authentication - not available with just an API key".into(),
            ));
        }

        Err(ChainError::ToolError(
            "Unknown Gmail command. Supported: list_emails, read_email, search_emails, send_email".into(),
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
