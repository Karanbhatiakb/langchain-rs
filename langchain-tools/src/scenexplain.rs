//! Scene explanation tool.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct SceneXplainTool {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for SceneXplainTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneXplainTool {
    pub fn new() -> Self {
        let api_key = std::env::var("SCENEXPLAIN_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://api.scenex.jina.ai/api/describe".into(),
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
impl BaseTool for SceneXplainTool {
    fn name(&self) -> &str {
        "scenexplain"
    }

    fn description(&self) -> &str {
        "Describe images using SceneXplain. Input should be an image URL."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let image_url = input.trim();
        if image_url.is_empty() {
            return Err(ChainError::ToolError("Empty image URL".into()));
        }
        if self.api_key.is_empty() {
            return Err(ChainError::ToolError("SCENEXPLAIN_API_KEY not set".into()));
        }
        let body = json!({
            "data": [image_url],
        });
        let resp = self
            .client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("SceneXplain API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "SceneXplain API returned {}: {}",
                status, text
            )));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        if let Some(descriptions) = result["results"].as_array() {
            let mut output = Vec::new();
            for desc in descriptions {
                if let Some(text) = desc["description"].as_str() {
                    output.push(text.to_string());
                }
            }
            if !output.is_empty() {
                return Ok(output.join("\n"));
            }
        }

        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?)
    }
}
