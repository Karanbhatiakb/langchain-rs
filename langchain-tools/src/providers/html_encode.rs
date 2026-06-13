//! HTML encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that HTML-encodes a string.
#[derive(Debug, Clone)]
pub struct HtmlEncodeTool;

impl HtmlEncodeTool {
    /// Create a new `HtmlEncodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlEncodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for HtmlEncodeTool {
    fn name(&self) -> &str {
        "html_encode"
    }

    fn description(&self) -> &str {
        "HTML-encodes the input string (escapes < > & \" ')."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HtmlEncodeTool is a stub");
        Ok("Result from html_encode".into())
    }
}
