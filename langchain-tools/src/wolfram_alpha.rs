//! Wolfram Alpha query tool.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

#[derive(Debug, Clone)]
pub struct WolframAlphaTool {
    app_id: String,
    client: reqwest::Client,
}

impl WolframAlphaTool {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BaseTool for WolframAlphaTool {
    fn name(&self) -> &str {
        "wolfram_alpha"
    }

    fn description(&self) -> &str {
        "Wolfram Alpha computational knowledge engine. Input should be a query string. Requires WOLFRAM_APP_ID env var. Uses the Wolfram Alpha Full Results API."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let query = input.trim();
        if query.is_empty() {
            return Err(ChainError::ToolError("Empty query".into()));
        }

        let url = format!(
            "https://api.wolframalpha.com/v2/query?input={}&appid={}&format=plaintext&output=json",
            urlencode(query),
            self.app_id
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("Wolfram Alpha API error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ChainError::ToolError(format!(
                "Wolfram Alpha API returned {}",
                resp.status()
            )));
        }

        let result: Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        let pods = result["queryresult"]["pods"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut output = Vec::new();
        for pod in pods {
            let title = pod["title"].as_str().unwrap_or("");
            let subpods = pod["subpods"].as_array().cloned().unwrap_or_default();
            for subpod in subpods {
                if let Some(text) = subpod["plaintext"].as_str() {
                    if !text.is_empty() {
                        output.push(format!("{}: {}", title, text));
                    }
                }
            }
        }

        if output.is_empty() {
            Ok("No results found".to_string())
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
