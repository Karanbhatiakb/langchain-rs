//! Data tool implementations.

pub use crate::database::{SQLDatabaseTool, ListTablesTool, QuerySQLTool};
pub use crate::requests::{RequestsGetTool, RequestsPostTool, RequestsPutTool, RequestsPatchTool, RequestsDeleteTool};

use async_trait::async_trait;
use crate::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct JSONAPITool {
    client: reqwest::Client,
    base_url: String,
    headers: Vec<(String, String)>,
}

impl JSONAPITool {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            headers: Vec::new(),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}

#[async_trait]
impl BaseTool for JSONAPITool {
    fn name(&self) -> &str {
        "json_api"
    }

    fn description(&self) -> &str {
        "Interact with a JSON API. Input format: METHOD path\\noptional_json_body"
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        let method_path = parts[0].trim();
        let mp: Vec<&str> = method_path.splitn(2, ' ').collect();
        if mp.len() < 2 {
            return Err(ChainError::ToolError("Usage: METHOD /path\\noptional_body".into()));
        }
        let method = mp[0].to_uppercase();
        let path = mp[1];
        let url = format!("{}{}", self.base_url, path);
        let mut req = match method.as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "PATCH" => self.client.patch(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(ChainError::ToolError(format!("Unsupported method: {}", method))),
        };
        for (k, v) in &self.headers {
            req = req.header(k.as_str(), v.as_str());
        }
        if parts.len() > 1 && !parts[1].trim().is_empty() && (method == "POST" || method == "PUT" || method == "PATCH") {
            let body: serde_json::Value = serde_json::from_str(parts[1].trim())
                .map_err(|e| ChainError::ToolError(format!("Invalid JSON body: {}", e)))?;
            req = req.json(&body);
        }
        let resp = req.send().await
            .map_err(|e| ChainError::ToolError(format!("Request failed: {}", e)))?;
        let status = resp.status();
        let body = resp.text().await
            .map_err(|e| ChainError::ToolError(format!("Read error: {}", e)))?;
        Ok(format!("{} {}", status, body))
    }
}

pub struct CSVTool;

#[async_trait]
impl BaseTool for CSVTool {
    fn name(&self) -> &str {
        "csv_query"
    }

    fn description(&self) -> &str {
        "Query a CSV file. Input format: file_path\\nquery_type (head, columns, count, search:term)"
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        if parts.len() < 2 {
            return Err(ChainError::ToolError("Usage: file_path\\nquery_type".into()));
        }
        let path = parts[0].trim();
        let query = parts[1].trim();
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| ChainError::ToolError(format!("Failed to read CSV: {}", e)))?;
        let mut rdr = csv::Reader::from_reader(content.as_bytes());
        match query {
            "head" => {
                let mut output = Vec::new();
                let headers = rdr.headers().map_err(|e| ChainError::ToolError(format!("CSV error: {}", e)))?;
                output.push(headers.iter().collect::<Vec<_>>().join(", "));
                for (i, record) in rdr.records().enumerate().take(5) {
                    let r = record.map_err(|e| ChainError::ToolError(format!("CSV error: {}", e)))?;
                    output.push(r.iter().collect::<Vec<_>>().join(", "));
                    let _ = i;
                }
                Ok(output.join("\n"))
            }
            "columns" => {
                let headers = rdr.headers().map_err(|e| ChainError::ToolError(format!("CSV error: {}", e)))?;
                Ok(headers.iter().collect::<Vec<_>>().join("\n"))
            }
            "count" => {
                let count = rdr.records().count();
                Ok(format!("{} rows", count))
            }
            q if q.starts_with("search:") => {
                let term = &q[7..];
                let mut output = Vec::new();
                for record in rdr.records() {
                    let r = record.map_err(|e| ChainError::ToolError(format!("CSV error: {}", e)))?;
                    if r.iter().any(|f| f.contains(term)) {
                        output.push(r.iter().collect::<Vec<_>>().join(", "));
                    }
                }
                if output.is_empty() {
                    Ok(format!("No rows matching '{}'", term))
                } else {
                    Ok(output.join("\n"))
                }
            }
            _ => Err(ChainError::ToolError("Unknown query. Use: head, columns, count, search:term".into())),
        }
    }
}
