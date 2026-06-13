//! Notion API tool.

use async_trait::async_trait;
use serde_json::{json, Value};

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct NotionTool {
    token: String,
    client: reqwest::Client,
}

impl NotionTool {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Notion-Version",
            reqwest::header::HeaderValue::from_static("2022-06-28"),
        );
        headers
    }
}

#[async_trait]
impl BaseTool for NotionTool {
    fn name(&self) -> &str {
        "notion"
    }

    fn description(&self) -> &str {
        "Notion API tool. Supports: list_databases, query_database <database_id>, get_page <page_id>, create_page <database_id>\\n<title>, update_page <page_id>\\n<properties_json>. Requires NOTION_TOKEN env var."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();

        if input == "list_databases" {
            let url = "https://api.notion.com/v1/search";
            let body = json!({"query": "", "filter": {"value": "database", "property": "object"}});
            let resp = self
                .client
                .post(url)
                .headers(self.headers())
                .json(&body)
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Notion API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Notion API returned {}", resp.status())));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let results = result["results"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for db in results {
                output.push(format!(
                    "{}: {}",
                    db["id"].as_str().unwrap_or(""),
                    db["title"][0]["plain_text"].as_str().unwrap_or("Unnamed"),
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("query_database ") {
            let db_id = cmd.trim();
            let url = format!("https://api.notion.com/v1/databases/{}/query", db_id);
            let resp = self
                .client
                .post(&url)
                .headers(self.headers())
                .json(&json!({}))
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Notion API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Notion API returned {}", resp.status())));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            let results = result["results"].as_array().cloned().unwrap_or_default();
            let mut output = Vec::new();
            for page in results {
                let title = page["properties"]["title"]["title"][0]["plain_text"]
                    .as_str()
                    .unwrap_or("Untitled");
                output.push(format!(
                    "{}: {}",
                    page["id"].as_str().unwrap_or(""),
                    title,
                ));
            }
            return Ok(output.join("\n"));
        }

        if let Some(cmd) = input.strip_prefix("get_page ") {
            let page_id = cmd.trim();
            let url = format!("https://api.notion.com/v1/pages/{}", page_id);
            let resp = self
                .client
                .get(&url)
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Notion API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Notion API returned {}", resp.status())));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            return Ok(serde_json::to_string_pretty(&result).unwrap_or_default());
        }

        if let Some(cmd) = input.strip_prefix("create_page ") {
            let parts: Vec<&str> = cmd.splitn(2, '\n').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError("Usage: create_page <database_id>\\n<title>".into()));
            }
            let db_id = parts[0].trim();
            let title = parts[1].trim();

            let body = json!({
                "parent": {"database_id": db_id},
                "properties": {
                    "title": {
                        "title": [{"text": {"content": title}}]
                    }
                }
            });

            let url = "https://api.notion.com/v1/pages";
            let resp = self
                .client
                .post(url)
                .headers(self.headers())
                .json(&body)
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Notion API error: {}", e)))?;

            if !resp.status().is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(ChainError::ToolError(format!("Notion API error: {}", text)));
            }

            let result: Value = resp.json().await.map_err(|e| ChainError::ToolError(format!("Parse error: {}", e)))?;
            return Ok(format!("Created page: {}", result["id"].as_str().unwrap_or("")));
        }

        if let Some(cmd) = input.strip_prefix("update_page ") {
            let parts: Vec<&str> = cmd.splitn(2, '\n').collect();
            if parts.len() < 2 {
                return Err(ChainError::ToolError("Usage: update_page <page_id>\\n<properties_json>".into()));
            }
            let page_id = parts[0].trim();
            let properties: Value = serde_json::from_str(parts[1].trim())
                .map_err(|e| ChainError::ToolError(format!("Invalid JSON: {}", e)))?;

            let body = json!({"properties": properties});
            let url = format!("https://api.notion.com/v1/pages/{}", page_id);
            let resp = self
                .client
                .patch(&url)
                .headers(self.headers())
                .json(&body)
                .send()
                .await
                .map_err(|e| ChainError::ToolError(format!("Notion API error: {}", e)))?;

            if !resp.status().is_success() {
                return Err(ChainError::ToolError(format!("Notion API returned {}", resp.status())));
            }

            Ok("Page updated successfully".to_string())
        } else {
            Err(ChainError::ToolError(
                "Unknown Notion command. Supported: list_databases, query_database, get_page, create_page, update_page".into(),
            ))
        }
    }
}
