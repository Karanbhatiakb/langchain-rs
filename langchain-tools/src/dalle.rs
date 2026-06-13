//! DALL-E image generation tool.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct DalleTool {
    api_key: String,
    base_url: String,
    model: String,
    size: String,
    client: reqwest::Client,
}

impl Default for DalleTool {
    fn default() -> Self {
        Self::new()
    }
}

impl DalleTool {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://api.openai.com/v1/images/generations".into(),
            model: "dall-e-3".into(),
            size: "1024x1024".into(),
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

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn with_size(mut self, size: &str) -> Self {
        self.size = size.to_string();
        self
    }
}

#[async_trait]
impl BaseTool for DalleTool {
    fn name(&self) -> &str {
        "dalle"
    }

    fn description(&self) -> &str {
        "Generate images using DALL-E. Input should be a text description of the desired image."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let prompt = input.trim();
        if prompt.is_empty() {
            return Err(ChainError::ToolError("Empty image prompt".into()));
        }
        if self.api_key.is_empty() {
            return Err(ChainError::ToolError("OPENAI_API_KEY not set".into()));
        }
        let body = json!({
            "model": self.model,
            "prompt": prompt,
            "n": 1,
            "size": self.size,
        });
        let resp = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("DALL-E API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!("DALL-E API returned {}: {}", status, text)));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        if let Some(images) = result["data"].as_array() {
            if let Some(first) = images.first() {
                let url = first["url"].as_str().unwrap_or("no url");
                let revised = first["revised_prompt"].as_str().unwrap_or("");
                return Ok(format!("Image URL: {}\nRevised prompt: {}", url, revised));
            }
        }
        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?)
    }
}
