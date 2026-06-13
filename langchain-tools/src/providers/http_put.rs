//! HTTP PUT tool implementation.
//!
//! Provides a `HttpPutTool` that sends an HTTP PUT request.
//! Gated behind the `http_put` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for sending HTTP PUT requests.
#[derive(Debug, Clone)]
pub struct HttpPutTool;

impl HttpPutTool {
    /// Create a new `HttpPutTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for HttpPutTool {
    fn name(&self) -> &str {
        "http_put"
    }

    fn description(&self) -> &str {
        "Send an HTTP PUT request to a URL with a JSON body. \
         Input should be '<url>\\n<body>'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HttpPutTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
