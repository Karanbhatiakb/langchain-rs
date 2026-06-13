//! HTTP DELETE tool implementation.
//!
//! Provides a `HttpDeleteTool` that sends an HTTP DELETE request.
//! Gated behind the `http_delete` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for sending HTTP DELETE requests.
#[derive(Debug, Clone)]
pub struct HttpDeleteTool;

impl HttpDeleteTool {
    /// Create a new `HttpDeleteTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for HttpDeleteTool {
    fn name(&self) -> &str {
        "http_delete"
    }

    fn description(&self) -> &str {
        "Send an HTTP DELETE request to a URL. \
         Input should be a fully qualified URL."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HttpDeleteTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
