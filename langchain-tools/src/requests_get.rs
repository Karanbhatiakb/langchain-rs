//! Tool for making HTTP GET requests.
//!
//! The [`RequestsGetTool`] sends an HTTP GET request to the provided URL and
//! returns the response body as a string.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

/// Tool that performs an HTTP GET request.
///
/// # Input format
///
/// ```text
/// <URL>
/// ```
///
/// # Stub
///
/// This is a stub implementation. Production use should replace the body of
/// [`invoke`](RequestsGetTool::invoke) with a real HTTP GET call.
#[derive(Debug, Clone)]
pub struct RequestsGetTool;

impl RequestsGetTool {
    /// Creates a new [`RequestsGetTool`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequestsGetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for RequestsGetTool {
    fn name(&self) -> &str {
        "requests_get"
    }

    fn description(&self) -> &str {
        "Makes an HTTP GET request to a URL. Input should be a URL string."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let url = input.trim();
        if url.is_empty() {
            return Err(ChainError::ToolError("URL must not be empty".into()));
        }
        Ok("Result from requests_get (stub)".to_string())
    }
}
