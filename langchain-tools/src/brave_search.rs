//! Brave Search tool.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct BraveSearchTool {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for BraveSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BraveSearchTool {
    pub fn new() -> Self {
        let api_key = std::env::var("BRAVE_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://api.search.brave.com/res/v1/web/search".into(),
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
impl BaseTool for BraveSearchTool {
    fn name(&self) -> &str {
        "brave_search"
    }

    fn description(&self) -> &str {
        "Search the web using Brave Search API. Input should be a search query string."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let query = input.trim();
        if query.is_empty() {
            return Err(ChainError::ToolError("Empty search query".into()));
        }
        if self.api_key.is_empty() {
            return Err(ChainError::ToolError("BRAVE_API_KEY not set".into()));
        }
        let url = format!("{}?q={}", self.base_url, urlencode(query));
        let resp = self
            .client
            .get(&url)
            .header("X-Subscription-Token", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Brave API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!("Brave API returned {}: {}", status, text)));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        let mut output = Vec::new();
        if let Some(results) = result["web"]["results"].as_array() {
            for (i, item) in results.iter().take(5).enumerate() {
                let title = item["title"].as_str().unwrap_or("");
                let url = item["url"].as_str().unwrap_or("");
                let desc = item["description"].as_str().unwrap_or("");
                output.push(format!("{}. {} - {}\n {}", i + 1, title, url, desc));
            }
        }
        if output.is_empty() {
            Ok("No results found".into())
        } else {
            Ok(output.join("\n"))
        }
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
