//! Playwright browser automation tool.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct PlaywrightTool {
    base_url: String,
    client: reqwest::Client,
}

impl Default for PlaywrightTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaywrightTool {
    pub fn new() -> Self {
        let base_url =
            std::env::var("PLAYWRIGHT_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".into());
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[async_trait]
impl BaseTool for PlaywrightTool {
    fn name(&self) -> &str {
        "playwright"
    }

    fn description(&self) -> &str {
        "Browser automation via Playwright. Supports: navigate <url>, click <selector>, screenshot, extract_text <selector>, fill <selector> <value>. Commands separated by newlines."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if let Some(url) = input.strip_prefix("navigate ") {
            let body = json!({ "action": "navigate", "url": url.trim() });
            return self.execute_command(&body).await;
        }

        if let Some(selector) = input.strip_prefix("click ") {
            let body = json!({ "action": "click", "selector": selector.trim() });
            return self.execute_command(&body).await;
        }

        if input == "screenshot" {
            let body = json!({ "action": "screenshot" });
            return self.execute_command(&body).await;
        }

        if let Some(selector) = input.strip_prefix("extract_text ") {
            let body = json!({ "action": "extract_text", "selector": selector.trim() });
            return self.execute_command(&body).await;
        }

        if let Some(rest) = input.strip_prefix("fill ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError(
                    "fill requires: fill <selector> <value>".into(),
                ));
            }
            let body = json!({
                "action": "fill",
                "selector": parts[0].trim(),
                "value": parts[1].trim(),
            });
            return self.execute_command(&body).await;
        }

        Err(ChainError::ToolError(
            "Unknown Playwright command. Supported: navigate, click, screenshot, extract_text, fill".into(),
        ))
    }
}

impl PlaywrightTool {
    async fn execute_command(&self, body: &serde_json::Value) -> ToolResult {
        let url = format!("{}/command", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Playwright server error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Playwright server returned {}: {}",
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
