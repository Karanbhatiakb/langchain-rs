//! GraphQL query tool.

use async_trait::async_trait;
use serde_json::json;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct GraphQLTool {
    endpoint: String,
    headers: Vec<(String, String)>,
    client: reqwest::Client,
}

impl Default for GraphQLTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphQLTool {
    pub fn new() -> Self {
        Self {
            endpoint: std::env::var("GRAPHQL_ENDPOINT").unwrap_or_default(),
            headers: Vec::new(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_endpoint(mut self, url: &str) -> Self {
        self.endpoint = url.to_string();
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.headers.push(("Authorization".to_string(), format!("Bearer {}", key)));
        self
    }
}

#[async_trait]
impl BaseTool for GraphQLTool {
    fn name(&self) -> &str {
        "graphql"
    }

    fn description(&self) -> &str {
        "Execute a GraphQL query. Input should be a GraphQL query string, optionally preceded by variables as JSON on the first line separated by a newline."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if input.is_empty() {
            return Err(ChainError::ToolError("Empty GraphQL query".into()));
        }
        if self.endpoint.is_empty() {
            return Err(ChainError::ToolError("GRAPHQL_ENDPOINT not set".into()));
        }

        let (variables, query) = if let Some(pos) = input.find('\n') {
            let vars_str = &input[..pos];
            let q = &input[pos + 1..];
            match serde_json::from_str::<serde_json::Value>(vars_str.trim()) {
                Ok(v) if v.is_object() => (v, q.trim().to_string()),
                _ => (json!({}), input.to_string()),
            }
        } else {
            (json!({}), input.to_string())
        };

        let body = json!({
            "query": query,
            "variables": variables,
        });

        let mut req = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json");

        for (key, value) in &self.headers {
            req = req.header(key.as_str(), value.as_str());
        }

        let resp = req
            .json(&body)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("GraphQL request error: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ChainError::ToolError(format!("GraphQL endpoint returned {}: {}", status, text)));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;

        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ChainError::ToolError(format!("Serialization error: {}", e)))?)
    }
}
