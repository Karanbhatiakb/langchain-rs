//! Tavily search tool.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct TavilySearchTool {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for TavilySearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl TavilySearchTool {
    pub fn new() -> Self {
        let api_key = std::env::var("TAVILY_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://api.tavily.com/search".into(),
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
impl BaseTool for TavilySearchTool {
    fn name(&self) -> &str {
        "tavily_search"
    }

    fn description(&self) -> &str {
        "Search the web using Tavily API. Input should be a search query string."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let query = input.trim();
        if query.is_empty() {
            return Err(ChainError::ToolError("Empty search query".into()));
        }
        if self.api_key.is_empty() {
            return Err(ChainError::ToolError("TAVILY_API_KEY not set".into()));
        }
        let body = json!({
            "api_key": self.api_key,
            "query": query,
            "max_results": 5,
        });
        let resp = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Tavily API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!("Tavily API returned {}: {}", status, text)));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        let mut output = Vec::new();
        if let Some(results) = result["results"].as_array() {
            for (i, item) in results.iter().enumerate() {
                let title = item["title"].as_str().unwrap_or("");
                let url = item["url"].as_str().unwrap_or("");
                let content = item["content"].as_str().unwrap_or("");
                output.push(format!("{}. {} - {}\n {}", i + 1, title, url, content));
            }
        }
        if output.is_empty() {
            Ok("No results found".into())
        } else {
            Ok(output.join("\n"))
        }
    }
}
