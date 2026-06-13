//! HTML decode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that HTML-decodes a string.
#[derive(Debug, Clone)]
pub struct HtmlDecodeTool;

impl HtmlDecodeTool {
    /// Create a new `HtmlDecodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlDecodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for HtmlDecodeTool {
    fn name(&self) -> &str {
        "html_decode"
    }

    fn description(&self) -> &str {
        "HTML-decodes the input string (unescapes HTML entities)."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HtmlDecodeTool is a stub");
        Ok("Result from html_decode".into())
    }
}
