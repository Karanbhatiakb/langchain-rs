//! HTTP POST tool implementation.
//!
//! Provides a `HttpPostTool` that sends an HTTP POST request.
//! Gated behind the `http_post` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for sending HTTP POST requests.
#[derive(Debug, Clone)]
pub struct HttpPostTool;

impl HttpPostTool {
    /// Create a new `HttpPostTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for HttpPostTool {
    fn name(&self) -> &str {
        "http_post"
    }

    fn description(&self) -> &str {
        "Send an HTTP POST request to a URL with a JSON body. \
         Input should be '<url>\\n<body>'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HttpPostTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
