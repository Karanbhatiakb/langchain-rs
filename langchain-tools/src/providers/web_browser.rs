//! Web browser tool implementation.
//!
//! Provides a `WebBrowserTool` that fetches and returns the text content of a
//! given URL. Gated behind the `web_browser` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for simplified web browsing — fetches page content from a URL.
#[derive(Debug, Clone)]
pub struct WebBrowserTool {
    user_agent: String,
}

impl WebBrowserTool {
    /// Create a new `WebBrowserTool` with the given user agent.
    pub fn new(user_agent: impl Into<String>) -> Self {
        Self {
            user_agent: user_agent.into(),
        }
    }
}

#[async_trait]
impl BaseTool for WebBrowserTool {
    fn name(&self) -> &str {
        "web_browser"
    }

    fn description(&self) -> &str {
        "Fetch and return the text content of a web page. Input should be a \
         fully qualified URL (e.g. https://example.com)."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("WebBrowserTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
