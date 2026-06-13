//! Coding tool implementations.

use async_trait::async_trait;
use crate::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct StackOverflowTool {
    client: reqwest::Client,
}

impl Default for StackOverflowTool {
    fn default() -> Self {
        Self::new()
    }
}

impl StackOverflowTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for StackOverflowTool {
    fn name(&self) -> &str {
        "stackoverflow"
    }

    fn description(&self) -> &str {
        "Search Stack Overflow. Input should be a search query. Returns top questions with accepted answers."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let query = input.trim();
        let url = format!(
            "https://api.stackexchange.com/2.3/search/advanced?order=desc&sort=relevance&q={}&site=stackoverflow&filter=withbody",
            urlencode(query)
        );
        let resp = self.client.get(&url)
            .send().await
            .map_err(|e| ChainError::ToolError(format!("StackOverflow API error: {}", e)))?;
        let result: serde_json::Value = resp.json().await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
        let mut output = Vec::new();
        if let Some(items) = result["items"].as_array() {
            for (i, item) in items.iter().take(3).enumerate() {
                let title = item["title"].as_str().unwrap_or("");
                let score = item["score"].as_i64().unwrap_or(0);
                let is_answered = item["is_answered"].as_bool().unwrap_or(false);
                let link = item["link"].as_str().unwrap_or("");
                output.push(format!("{}. {} [score={}, answered={}] {}", i + 1, title, score, is_answered, link));
            }
        }
        if output.is_empty() {
            Ok("No results found".into())
        } else {
            Ok(output.join("\n"))
        }
    }
}

pub struct GitHubSearchTool {
    client: reqwest::Client,
    token: Option<String>,
}

impl GitHubSearchTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }
}

#[async_trait]
impl BaseTool for GitHubSearchTool {
    fn name(&self) -> &str {
        "github_search"
    }

    fn description(&self) -> &str {
        "Search GitHub repositories, code, or issues. Input format: <type> <query> where type is repos, code, or issues."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        let (search_type, query) = if let Some(rest) = input.strip_prefix("repos ") {
            ("repositories", rest)
        } else if let Some(rest) = input.strip_prefix("code ") {
            ("code", rest)
        } else if let Some(rest) = input.strip_prefix("issues ") {
            ("issues", rest)
        } else {
            ("repositories", input)
        };
        let url = format!("https://api.github.com/search/{}?q={}&per_page=5", search_type, urlencode(query));
        let mut req = self.client.get(&url)
            .header("User-Agent", "langchain-rs");
        if let Some(ref token) = self.token {
            req = req.header("Authorization", format!("token {}", token));
        }
        let resp = req.send().await
            .map_err(|e| ChainError::ToolError(format!("GitHub API error: {}", e)))?;
        let result: serde_json::Value = resp.json().await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
        let mut output = Vec::new();
        if let Some(items) = result["items"].as_array() {
            for (i, item) in items.iter().take(5).enumerate() {
                let name = item["full_name"].as_str().or_else(|| item["name"].as_str()).unwrap_or("");
                let desc = item["description"].as_str().unwrap_or("");
                let url = item["html_url"].as_str().unwrap_or("");
                output.push(format!("{}. {} - {}\n   {}", i + 1, name, desc, url));
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
