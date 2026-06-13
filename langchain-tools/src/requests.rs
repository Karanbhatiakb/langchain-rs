//! HTTP requests tool.

use async_trait::async_trait;
use serde_json::Value;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct RequestsWrapper {
    client: reqwest::Client,
}

impl Default for RequestsWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestsWrapper {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn get(&self, url: &str) -> Result<String, ChainError> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("GET request failed: {}", e)))?;
        Ok(resp
            .text()
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?)
    }

    pub async fn post(&self, url: &str, body: Option<Value>) -> Result<String, ChainError> {
        let mut req = self.client.post(url);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("POST request failed: {}", e)))?;
        Ok(resp
            .text()
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?)
    }

    pub async fn put(&self, url: &str, body: Option<Value>) -> Result<String, ChainError> {
        let mut req = self.client.put(url);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("PUT request failed: {}", e)))?;
        Ok(resp
            .text()
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?)
    }

    pub async fn patch(&self, url: &str, body: Option<Value>) -> Result<String, ChainError> {
        let mut req = self.client.patch(url);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("PATCH request failed: {}", e)))?;
        Ok(resp
            .text()
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?)
    }

    pub async fn delete(&self, url: &str) -> Result<String, ChainError> {
        let resp = self
            .client
            .delete(url)
            .send()
            .await
            .map_err(|e| ChainError::ToolError(format!("DELETE request failed: {}", e)))?;
        Ok(resp
            .text()
            .await
            .map_err(|e| ChainError::ToolError(format!("Failed to read response: {}", e)))?)
    }
}

pub struct RequestsGetTool {
    wrapper: RequestsWrapper,
}

impl RequestsGetTool {
    pub fn new() -> Self {
        Self {
            wrapper: RequestsWrapper::new(),
        }
    }

    pub fn with_wrapper(wrapper: RequestsWrapper) -> Self {
        Self { wrapper }
    }
}

#[async_trait]
impl BaseTool for RequestsGetTool {
    fn name(&self) -> &str {
        "requests_get"
    }

    fn description(&self) -> &str {
        "Makes a GET request to a URL. Input should be a URL."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        self.wrapper.get(input.trim()).await
    }
}

pub struct RequestsPostTool {
    wrapper: RequestsWrapper,
}

impl RequestsPostTool {
    pub fn new() -> Self {
        Self {
            wrapper: RequestsWrapper::new(),
        }
    }

    pub fn with_wrapper(wrapper: RequestsWrapper) -> Self {
        Self { wrapper }
    }
}

#[async_trait]
impl BaseTool for RequestsPostTool {
    fn name(&self) -> &str {
        "requests_post"
    }

    fn description(&self) -> &str {
        "Makes a POST request to a URL. Input should be a URL followed by a newline and optional JSON body."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        let url = parts[0].trim();
        let body = if parts.len() > 1 {
            Some(
                serde_json::from_str::<Value>(parts[1].trim())
                    .map_err(|e| ChainError::ToolError(format!("Invalid JSON body: {}", e)))?,
            )
        } else {
            None
        };
        self.wrapper.post(url, body).await
    }
}

pub struct RequestsPutTool {
    wrapper: RequestsWrapper,
}

impl RequestsPutTool {
    pub fn new() -> Self {
        Self {
            wrapper: RequestsWrapper::new(),
        }
    }

    pub fn with_wrapper(wrapper: RequestsWrapper) -> Self {
        Self { wrapper }
    }
}

#[async_trait]
impl BaseTool for RequestsPutTool {
    fn name(&self) -> &str {
        "requests_put"
    }

    fn description(&self) -> &str {
        "Makes a PUT request to a URL. Input should be a URL followed by a newline and optional JSON body."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        let url = parts[0].trim();
        let body = if parts.len() > 1 {
            Some(
                serde_json::from_str::<Value>(parts[1].trim())
                    .map_err(|e| ChainError::ToolError(format!("Invalid JSON body: {}", e)))?,
            )
        } else {
            None
        };
        self.wrapper.put(url, body).await
    }
}

pub struct RequestsPatchTool {
    wrapper: RequestsWrapper,
}

impl RequestsPatchTool {
    pub fn new() -> Self {
        Self {
            wrapper: RequestsWrapper::new(),
        }
    }

    pub fn with_wrapper(wrapper: RequestsWrapper) -> Self {
        Self { wrapper }
    }
}

#[async_trait]
impl BaseTool for RequestsPatchTool {
    fn name(&self) -> &str {
        "requests_patch"
    }

    fn description(&self) -> &str {
        "Makes a PATCH request to a URL. Input should be a URL followed by a newline and optional JSON body."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let parts: Vec<&str> = input.splitn(2, '\n').collect();
        let url = parts[0].trim();
        let body = if parts.len() > 1 {
            Some(
                serde_json::from_str::<Value>(parts[1].trim())
                    .map_err(|e| ChainError::ToolError(format!("Invalid JSON body: {}", e)))?,
            )
        } else {
            None
        };
        self.wrapper.patch(url, body).await
    }
}

pub struct RequestsDeleteTool {
    wrapper: RequestsWrapper,
}

impl RequestsDeleteTool {
    pub fn new() -> Self {
        Self {
            wrapper: RequestsWrapper::new(),
        }
    }

    pub fn with_wrapper(wrapper: RequestsWrapper) -> Self {
        Self { wrapper }
    }
}

#[async_trait]
impl BaseTool for RequestsDeleteTool {
    fn name(&self) -> &str {
        "requests_delete"
    }

    fn description(&self) -> &str {
        "Makes a DELETE request to a URL. Input should be a URL."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        self.wrapper.delete(input.trim()).await
    }
}
