//! SerpAPI search tool.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct SerpApiTool {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for SerpApiTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SerpApiTool {
    pub fn new() -> Self {
        let api_key = std::env::var("SERPAPI_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://serpapi.com/search".into(),
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
impl BaseTool for SerpApiTool {
    fn name(&self) -> &str {
        "serpapi"
    }

    fn description(&self) -> &str {
        "Search Google using SerpAPI. Input should be a search query string."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let query = input.trim();
        if query.is_empty() {
            return Err(ChainError::ToolError("Empty search query".into()));
        }
        if self.api_key.is_empty() {
            return Err(ChainError::ToolError("SERPAPI_API_KEY not set".into()));
        }
        let url = format!(
            "{}?q={}&api_key={}&engine=google",
            self.base_url,
            urlencode(query),
            self.api_key
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("SerpAPI error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!("SerpAPI returned {}: {}", status, text)));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        let mut output = Vec::new();
        if let Some(answer) = result["answer_box"]["snippet"].as_str() {
            output.push(format!("Answer: {}", answer));
        }
        if let Some(results) = result["organic_results"].as_array() {
            for (i, item) in results.iter().take(5).enumerate() {
                let title = item["title"].as_str().unwrap_or("");
                let link = item["link"].as_str().unwrap_or("");
                let snippet = item["snippet"].as_str().unwrap_or("");
                output.push(format!("{}. {} - {}\n {}", i + 1, title, link, snippet));
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
