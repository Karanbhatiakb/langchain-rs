//! Zendesk support tool.

use async_trait::async_trait;
use base64::Engine;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct ZendeskTool {
    api_key: String,
    email: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for ZendeskTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ZendeskTool {
    pub fn new() -> Self {
        let api_key = std::env::var("ZENDESK_API_KEY").unwrap_or_default();
        let email = std::env::var("ZENDESK_EMAIL").unwrap_or_default();
        let base_url = std::env::var("ZENDESK_BASE_URL")
            .unwrap_or_else(|_| "https://example.zendesk.com/api/v2".into());
        Self {
            api_key,
            email,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = key.to_string();
        self
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    fn auth_header(&self) -> String {
        let creds = format!("{}/token:{}", self.email, self.api_key);
        format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(creds))
    }
}

#[async_trait]
impl BaseTool for ZendeskTool {
    fn name(&self) -> &str {
        "zendesk"
    }

    fn description(&self) -> &str {
        "Zendesk API tool. Supports: list_tickets, get_ticket <id>, create_ticket <JSON>, update_ticket <id> <JSON>, search <query>."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if input.is_empty() {
            return Err(ChainError::ToolError("Empty Zendesk command".into()));
        }
        if self.api_key.is_empty() {
            return Err(ChainError::ToolError("ZENDESK_API_KEY not set".into()));
        }

        if input == "list_tickets" {
            let url = format!("{}/tickets", self.base_url);
            return self.get_json(&url).await;
        }

        if let Some(id) = input.strip_prefix("get_ticket ") {
            let url = format!("{}/tickets/{}", self.base_url, id.trim());
            return self.get_json(&url).await;
        }

        if let Some(json_str) = input.strip_prefix("create_ticket ") {
            let data: serde_json::Value = serde_json::from_str(json_str.trim())
                .map_err(|e| ChainError::ToolError(format!("Invalid JSON: {}", e)))?;
            let url = format!("{}/tickets", self.base_url);
            let body = json!({ "ticket": data });
            return self.post_json(&url, &body).await;
        }

        if let Some(rest) = input.strip_prefix("update_ticket ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError(
                    "update_ticket requires: update_ticket <id> <JSON>".into(),
                ));
            }
            let id = parts[0].trim();
            let data: serde_json::Value = serde_json::from_str(parts[1].trim())
                .map_err(|e| ChainError::ToolError(format!("Invalid JSON: {}", e)))?;
            let url = format!("{}/tickets/{}", self.base_url, id);
            let body = json!({ "ticket": data });
            return self.put_json(&url, &body).await;
        }

        if let Some(query) = input.strip_prefix("search ") {
            let url = format!(
                "{}/search.json?query={}",
                self.base_url,
                urlencode(query.trim())
            );
            return self.get_json(&url).await;
        }

        Err(ChainError::ToolError(
            "Unknown Zendesk command. Supported: list_tickets, get_ticket, create_ticket, update_ticket, search".into(),
        ))
    }
}

impl ZendeskTool {
    async fn get_json(&self, url: &str) -> ToolResult {
        let resp = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Zendesk API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Zendesk API returned {}: {}",
                status, text
            )));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?)
    }

    async fn post_json(&self, url: &str, body: &serde_json::Value) -> ToolResult {
        let resp = self
            .client
            .post(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Zendesk API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Zendesk API returned {}: {}",
                status, text
            )));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?)
    }

    async fn put_json(&self, url: &str, body: &serde_json::Value) -> ToolResult {
        let resp = self
            .client
            .put(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Zendesk API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Zendesk API returned {}: {}",
                status, text
            )));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?)
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
