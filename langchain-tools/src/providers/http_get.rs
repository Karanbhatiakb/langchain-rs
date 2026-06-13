//! HTTP GET tool implementation.
//!
//! Provides a `HttpGetTool` that sends an HTTP GET request.
//! Gated behind the `http_get` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for sending HTTP GET requests.
#[derive(Debug, Clone)]
pub struct HttpGetTool;

impl HttpGetTool {
    /// Create a new `HttpGetTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for HttpGetTool {
    fn name(&self) -> &str {
        "http_get"
    }

    fn description(&self) -> &str {
        "Send an HTTP GET request to a URL and return the response. \
         Input should be a fully qualified URL."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HttpGetTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
