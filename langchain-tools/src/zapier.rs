//! Zapier integration tool.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct ZapierNlaTool {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for ZapierNlaTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ZapierNlaTool {
    pub fn new() -> Self {
        let api_key = std::env::var("ZAPIER_NLA_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://actions.zapier.com/api/v1/dynamic/execute/".into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = key.to_string();
        self
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[async_trait]
impl BaseTool for ZapierNlaTool {
    fn name(&self) -> &str {
        "zapier_nla"
    }

    fn description(&self) -> &str {
        "Execute Zapier Natural Language Actions. Input should be a natural language description of the action to perform."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let instructions = input.trim();
        if instructions.is_empty() {
            return Err(ChainError::ToolError("Empty action instruction".into()));
        }
        if self.api_key.is_empty() {
            return Err(ChainError::ToolError("ZAPIER_NLA_API_KEY not set".into()));
        }
        let body = json!({
            "instructions": instructions,
            "api_key": self.api_key,
        });
        let resp = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Zapier API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!("Zapier API returned {}: {}", status, text)));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?)
    }
}
