//! HTTP PATCH tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that sends an HTTP PATCH request.
#[derive(Debug, Clone)]
pub struct HttpPatchTool;

impl HttpPatchTool {
    /// Create a new `HttpPatchTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpPatchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for HttpPatchTool {
    fn name(&self) -> &str {
        "http_patch"
    }

    fn description(&self) -> &str {
        "Sends an HTTP PATCH request to a given URL."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HttpPatchTool is a stub");
        Ok("Result from http_patch".into())
    }
}
