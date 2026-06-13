//! Polygon.io financial data tool.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct PolygonTool {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl Default for PolygonTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PolygonTool {
    pub fn new() -> Self {
        let api_key = std::env::var("POLYGON_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://api.polygon.io".into(),
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
impl BaseTool for PolygonTool {
    fn name(&self) -> &str {
        "polygon"
    }

    fn description(&self) -> &str {
        "Polygon.io financial data API. Supports: aggregates <ticker> <from> <to>, ticker <symbol>, previous_close <ticker>. Dates in YYYY-MM-DD format."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if input.is_empty() {
            return Err(ChainError::ToolError("Empty Polygon command".into()));
        }
        if self.api_key.is_empty() {
            return Err(ChainError::ToolError("POLYGON_API_KEY not set".into()));
        }

        let parts: Vec<&str> = input.splitn(4, ' ').collect();
        match parts.first().map(|s| *s) {
            Some("aggregates") => {
                if parts.len() < 4 {
                    return Err(ChainError::ToolError(
                        "aggregates requires: aggregates <ticker> <from_date> <to_date>".into(),
                    ));
                }
                let ticker = parts[1];
                let from_date = parts[2];
                let to_date = parts[3];
                let url = format!(
                    "{}/v2/aggs/ticker/{}/range/1/day/{}/{}?apiKey={}",
                    self.base_url, ticker, from_date, to_date, self.api_key
                );
                self.get_json(&url).await
            }
            Some("ticker") => {
                if parts.len() < 2 {
                    return Err(ChainError::ToolError("ticker requires: ticker <symbol>".into()));
                }
                let symbol = parts[1];
                let url = format!(
                    "{}/v3/reference/tickers/{}?apiKey={}",
                    self.base_url, symbol, self.api_key
                );
                self.get_json(&url).await
            }
            Some("previous_close") => {
                if parts.len() < 2 {
                    return Err(ChainError::ToolError(
                        "previous_close requires: previous_close <ticker>".into(),
                    ));
                }
                let ticker = parts[1];
                let url = format!(
                    "{}/v2/aggs/ticker/{}/prev?apiKey={}",
                    self.base_url, ticker, self.api_key
                );
                self.get_json(&url).await
            }
            _ => Err(ChainError::ToolError(
                "Unknown Polygon command. Supported: aggregates, ticker, previous_close".into(),
            )),
        }
    }
}

impl PolygonTool {
    async fn get_json(&self, url: &str) -> ToolResult {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Polygon API error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!(
                "Polygon API returned {}: {}",
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
