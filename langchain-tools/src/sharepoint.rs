//! SharePoint tool.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct SharePointTool {
    access_token: String,
    base_url: String,
    site_id: String,
    client: reqwest::Client,
}

impl Default for SharePointTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SharePointTool {
    pub fn new() -> Self {
        let access_token = std::env::var("SHAREPOINT_ACCESS_TOKEN").unwrap_or_default();
        let site_id = std::env::var("SHAREPOINT_SITE_ID").unwrap_or_default();
        Self {
            access_token,
            base_url: "https://graph.microsoft.com/v1.0".into(),
            site_id,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_access_token(mut self, token: &str) -> Self {
        self.access_token = token.to_string();
        self
    }

    pub fn with_site_id(mut self, id: &str) -> Self {
        self.site_id = id.to_string();
        self
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[async_trait]
impl BaseTool for SharePointTool {
    fn name(&self) -> &str {
        "sharepoint"
    }

    fn description(&self) -> &str {
        "SharePoint API via Microsoft Graph. Supports: list_drives, list_items <drive_id>, get_item <drive_id> <item_id>, search <query>."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if input.is_empty() {
            return Err(ChainError::ToolError("Empty SharePoint command".into()));
        }
        if self.access_token.is_empty() {
            return Err(ChainError::ToolError("SHAREPOINT_ACCESS_TOKEN not set".into()));
        }

        if input == "list_drives" {
            let url = format!("{}/sites/{}/drives", self.base_url, self.site_id);
            return self.get_json(&url).await;
        }

        if let Some(drive_id) = input.strip_prefix("list_items ") {
            let url = format!(
                "{}/drives/{}/root/children",
                self.base_url,
                drive_id.trim()
            );
            return self.get_json(&url).await;
        }

        if let Some(rest) = input.strip_prefix("get_item ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError(
                    "get_item requires: get_item <drive_id> <item_id>".into(),
                ));
            }
            let url = format!(
                "{}/drives/{}/items/{}",
                self.base_url,
                parts[0].trim(),
                parts[1].trim()
            );
            return self.get_json(&url).await;
        }

        if let Some(query) = input.strip_prefix("search ") {
            let url = format!(
                "{}/sites/{}/drive/root/search(q='{}')",
                self.base_url,
                self.site_id,
                urlencode(query.trim())
            );
            return self.get_json(&url).await;
        }

        Err(ChainError::ToolError(
            "Unknown SharePoint command. Supported: list_drives, list_items, get_item, search".into(),
        ))
    }
}

impl SharePointTool {
    async fn get_json(&self, url: &str) -> ToolResult {
        let resp = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("SharePoint API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "SharePoint API returned {}: {}",
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
