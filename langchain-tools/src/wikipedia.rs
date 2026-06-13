//! Wikipedia search tool.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct WikipediaTool {
    client: reqwest::Client,
}

impl Default for WikipediaTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WikipediaTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for WikipediaTool {
    fn name(&self) -> &str {
        "wikipedia"
    }

    fn description(&self) -> &str {
        "Wikipedia API tool. Supports: search <query>, summary <title>, page_content <title>, random_page. Uses the Wikipedia REST API."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if let Some(cmd) = input.strip_prefix("search ") {
            let query = cmd.trim();
            let url = format!(
                "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&srlimit=10",
                urlencode(query)
            );
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Wikipedia API error: {}", e)))?;

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let results = result["query"]["search"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for item in results {
                output.push(format!(
                    "{} - {}",
                    item["title"].as_str().unwrap_or(""),
                    item["snippet"].as_str().unwrap_or("").replace("<span class=\"searchmatch\">", "").replace("</span>", ""),
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("summary ") {
            let title = cmd.trim();
            let url = format!(
                "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
                urlencode(title)
            );
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Wikipedia API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Wikipedia returned {}", resp.status())));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            return Ok(format!(
                "Title: {}\nExtract: {}",
                result["title"].as_str().unwrap_or(""),
                result["extract"].as_str().unwrap_or(""),
            ));
        }

        if let Some(cmd) = input.strip_prefix("page_content ") {
            let title = cmd.trim();
            let url = format!(
                "https://en.wikipedia.org/w/api.php?action=query&titles={}&prop=extracts&explaintext=true&format=json",
                urlencode(title)
            );
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Wikipedia API error: {}", e)))?;

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let pages = result["query"]["pages"].as_object().cloned().unwrap_or_default();
            let mut content = String::new();
            for (_, page) in pages {
                content = page["extract"].as_str().unwrap_or("").to_string();
                break;
            }
            return Ok(content);
        }

        if input == "random_page" {
            let url = "https://en.wikipedia.org/api/rest_v1/page/random/summary";
            let resp = self
                .client
                .get(url)
                .header("User-Agent", "langchain-rs/1.0")
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Wikipedia API error: {}", e)))?;

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            return Ok(format!(
                "Title: {}\nExtract: {}",
                result["title"].as_str().unwrap_or(""),
                result["extract"].as_str().unwrap_or(""),
            ));
        }

        Err(ChainError::ToolError(
            "Unknown Wikipedia command. Supported: search, summary, page_content, random_page".into(),
        ))
    }
}

fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => '_'.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
