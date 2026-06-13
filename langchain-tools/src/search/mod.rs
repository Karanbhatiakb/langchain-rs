//! Search tool implementations (tavily, google, bing, etc.).

pub use crate::wikipedia::WikipediaTool;
pub use crate::arxiv::ArxivTool;
pub use crate::pubmed::PubMedTool;
pub use crate::wolfram_alpha::WolframAlphaTool;

use async_trait::async_trait;
use crate::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct DuckDuckGoSearchTool {
    client: reqwest::Client,
}

impl Default for DuckDuckGoSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl DuckDuckGoSearchTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for DuckDuckGoSearchTool {
    fn name(&self) -> &str {
        "duckduckgo_search"
    }

    fn description(&self) -> &str {
        "Search the web using DuckDuckGo. Input should be a search query string."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let query = input.trim();
        if query.is_empty() {
            return Err(ChainError::ToolError("Empty search query".into()));
        }
        let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1", urlencode(query));
        let resp = self.client.get(&url)
            .send().await
            .map_err(|e| ChainError::ToolError(format!("Search request failed: {}", e)))?;
        let result: serde_json::Value = resp.json().await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
        let abstract_text = result["Abstract"].as_str().unwrap_or("");
        let heading = result["Heading"].as_str().unwrap_or("Unknown");
        if abstract_text.is_empty() {
            Ok(format!("No instant answer found for '{}'", query))
        } else {
            Ok(format!("{}: {}", heading, abstract_text))
        }
    }
}

pub struct SearxSearchTool {
    base_url: String,
    client: reqwest::Client,
}

impl SearxSearchTool {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for SearxSearchTool {
    fn name(&self) -> &str {
        "searx_search"
    }

    fn description(&self) -> &str {
        "Search using a Searx instance. Input should be a search query string."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let query = input.trim();
        let url = format!("{}/search?q={}&format=json", self.base_url, urlencode(query));
        let resp = self.client.get(&url)
            .send().await
            .map_err(|e| ChainError::ToolError(format!("Searx request failed: {}", e)))?;
        let result: serde_json::Value = resp.json().await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
        let mut output = Vec::new();
        if let Some(results) = result["results"].as_array() {
            for (i, item) in results.iter().take(5).enumerate() {
                let title = item["title"].as_str().unwrap_or("");
                let url = item["url"].as_str().unwrap_or("");
                let snippet = item["content"].as_str().unwrap_or("");
                output.push(format!("{}. {} - {}\n   {}", i + 1, title, url, snippet));
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
