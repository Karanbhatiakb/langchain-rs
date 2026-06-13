//! Salesforce CRM tool.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct SalesforceTool {
    access_token: String,
    instance_url: String,
    client: reqwest::Client,
}

impl Default for SalesforceTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SalesforceTool {
    pub fn new() -> Self {
        let access_token = std::env::var("SF_ACCESS_TOKEN").unwrap_or_default();
        let instance_url = std::env::var("SF_INSTANCE_URL")
            .unwrap_or_else(|_| "https://login.salesforce.com".into());
        Self {
            access_token,
            instance_url,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_access_token(mut self, token: &str) -> Self {
        self.access_token = token.to_string();
        self
    }

    pub fn with_instance_url(mut self, url: &str) -> Self {
        self.instance_url = url.to_string();
        self
    }
}

#[async_trait]
impl BaseTool for SalesforceTool {
    fn name(&self) -> &str {
        "salesforce"
    }

    fn description(&self) -> &str {
        "Salesforce CRM API tool. Supports: query <SOQL>, get <object> <id>, create <object> <JSON>, update <object> <id> <JSON>, delete <object> <id>."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if input.is_empty() {
            return Err(ChainError::ToolError("Empty Salesforce command".into()));
        }
        if self.access_token.is_empty() {
            return Err(ChainError::ToolError("SF_ACCESS_TOKEN not set".into()));
        }

        if let Some(soql) = input.strip_prefix("query ") {
            let url = format!(
                "{}/services/data/v58.0/query/?q={}",
                self.instance_url,
                urlencode(soql.trim())
            );
            return self.get_json(&url).await;
        }

        if let Some(rest) = input.strip_prefix("get ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError("get requires: get <object> <id>".into()));
            }
            let url = format!(
                "{}/services/data/v58.0/sobjects/{}/{}",
                self.instance_url,
                parts[0].trim(),
                parts[1].trim()
            );
            return self.get_json(&url).await;
        }

        if let Some(rest) = input.strip_prefix("create ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError("create requires: create <object> <JSON>".into()));
            }
            let obj = parts[0].trim();
            let data: serde_json::Value = serde_json::from_str(parts[1].trim())
                .map_err(|e| ChainError::ToolError(format!("Invalid JSON: {}", e)))?;
            let url = format!(
                "{}/services/data/v58.0/sobjects/{}",
                self.instance_url, obj
            );
            return self.post_json(&url, &data).await;
        }

        if let Some(rest) = input.strip_prefix("update ") {
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            if parts.len() < 3 {
                return Err(ChainError::ToolError(
                    "update requires: update <object> <id> <JSON>".into(),
                ));
            }
            let obj = parts[0].trim();
            let id = parts[1].trim();
            let data: serde_json::Value = serde_json::from_str(parts[2].trim())
                .map_err(|e| ChainError::ToolError(format!("Invalid JSON: {}", e)))?;
            let url = format!(
                "{}/services/data/v58.0/sobjects/{}/{}",
                self.instance_url, obj, id
            );
            return self.patch_json(&url, &data).await;
        }

        if let Some(rest) = input.strip_prefix("delete ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError("delete requires: delete <object> <id>".into()));
            }
            let url = format!(
                "{}/services/data/v58.0/sobjects/{}/{}",
                self.instance_url,
                parts[0].trim(),
                parts[1].trim()
            );
            return self.delete_url(&url).await;
        }

        Err(ChainError::ToolError(
            "Unknown Salesforce command. Supported: query, get, create, update, delete".into(),
        ))
    }
}

impl SalesforceTool {
    async fn get_json(&self, url: &str) -> ToolResult {
        let resp = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Salesforce API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Salesforce API returned {}: {}",
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
            .map_err(|e| ChainError::ToolError(format!("Salesforce API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Salesforce API returned {}: {}",
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

    async fn patch_json(&self, url: &str, body: &serde_json::Value) -> ToolResult {
        let resp = self
            .client
            .patch(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Salesforce API error: {}", e)))?;

        if !resp.status().is_success() && resp.status().as_u16() != 204 {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Salesforce API returned {}: {}",
                status, text
            )));
        }

        Ok("OK".into())
    }

    async fn delete_url(&self, url: &str) -> ToolResult {
        let resp = self
            .client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Salesforce API error: {}", e)))?;

        if !resp.status().is_success() && resp.status().as_u16() != 204 {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Salesforce API returned {}: {}",
                status, text
            )));
        }

        Ok("OK".into())
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
