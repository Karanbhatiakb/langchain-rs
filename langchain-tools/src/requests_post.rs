//! Tool for making HTTP POST requests.
//!
//! The [`RequestsPostTool`] sends an HTTP POST request to the provided URL
//! with an optional JSON body and returns the response body as a string.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};

/// Tool that performs an HTTP POST request with an optional JSON body.
///
/// # Input format
///
/// ```text
/// <URL>
/// <optional JSON body>
/// ```
///
/// # Stub
///
/// This is a stub implementation. Production use should replace the body of
/// [`invoke`](RequestsPostTool::invoke) with a real HTTP POST call.
#[derive(Debug, Clone)]
pub struct RequestsPostTool;

impl RequestsPostTool {
    /// Creates a new [`RequestsPostTool`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequestsPostTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for RequestsPostTool {
    fn name(&self) -> &str {
        "requests_post"
    }

    fn description(&self) -> &str {
        "Makes an HTTP POST request to a URL with an optional JSON body. \
         Input should be a URL optionally followed by a newline and a JSON body."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from requests_post (stub)".to_string())
    }
}
