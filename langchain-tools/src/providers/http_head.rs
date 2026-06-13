//! HTTP HEAD tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that sends an HTTP HEAD request.
#[derive(Debug, Clone)]
pub struct HttpHeadTool;

impl HttpHeadTool {
    /// Create a new `HttpHeadTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpHeadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for HttpHeadTool {
    fn name(&self) -> &str {
        "http_head"
    }

    fn description(&self) -> &str {
        "Sends an HTTP HEAD request to a given URL."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HttpHeadTool is a stub");
        Ok("Result from http_head".into())
    }
}
