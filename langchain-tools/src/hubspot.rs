//! HubSpot CRM tool.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct HubSpotTool {
    access_token: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for HubSpotTool {
    fn default() -> Self {
        Self::new()
    }
}

impl HubSpotTool {
    pub fn new() -> Self {
        let access_token = std::env::var("HUBSPOT_ACCESS_TOKEN").unwrap_or_default();
        Self {
            access_token,
            base_url: "https://api.hubapi.com".into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_access_token(mut self, token: &str) -> Self {
        self.access_token = token.to_string();
        self
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[async_trait]
impl BaseTool for HubSpotTool {
    fn name(&self) -> &str {
        "hubspot"
    }

    fn description(&self) -> &str {
        "HubSpot CRM API tool. Supports: list_contacts, get_contact <id>, create_contact <JSON>, list_deals, get_deal <id>, search <object> <query>."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if input.is_empty() {
            return Err(ChainError::ToolError("Empty HubSpot command".into()));
        }
        if self.access_token.is_empty() {
            return Err(ChainError::ToolError("HUBSPOT_ACCESS_TOKEN not set".into()));
        }

        if input == "list_contacts" {
            let url = format!("{}/crm/v3/objects/contacts", self.base_url);
            return self.get_json(&url).await;
        }

        if let Some(id) = input.strip_prefix("get_contact ") {
            let url = format!(
                "{}/crm/v3/objects/contacts/{}",
                self.base_url,
                id.trim()
            );
            return self.get_json(&url).await;
        }

        if let Some(json_str) = input.strip_prefix("create_contact ") {
            let data: serde_json::Value = serde_json::from_str(json_str.trim())
                .map_err(|e| ChainError::ToolError(format!("Invalid JSON: {}", e)))?;
            let url = format!("{}/crm/v3/objects/contacts", self.base_url);
            let body = json!({ "properties": data });
            return self.post_json(&url, &body).await;
        }

        if input == "list_deals" {
            let url = format!("{}/crm/v3/objects/deals", self.base_url);
            return self.get_json(&url).await;
        }

        if let Some(id) = input.strip_prefix("get_deal ") {
            let url = format!(
                "{}/crm/v3/objects/deals/{}",
                self.base_url,
                id.trim()
            );
            return self.get_json(&url).await;
        }

        if let Some(rest) = input.strip_prefix("search ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError(
                    "search requires: search <object> <query>".into(),
                ));
            }
            let obj = parts[0].trim();
            let query = parts[1].trim();
            let url = format!("{}/crm/v3/objects/{}/search", self.base_url, obj);
            let body = json!({
                "query": query,
                "limit": 5,
            });
            return self.post_json(&url, &body).await;
        }

        Err(ChainError::ToolError(
            "Unknown HubSpot command. Supported: list_contacts, get_contact, create_contact, list_deals, get_deal, search".into(),
        ))
    }
}

impl HubSpotTool {
    async fn get_json(&self, url: &str) -> ToolResult {
        let resp = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("HubSpot API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "HubSpot API returned {}: {}",
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
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("HubSpot API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "HubSpot API returned {}: {}",
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
