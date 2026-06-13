//! String format tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that formats a string with given arguments.
#[derive(Debug, Clone)]
pub struct FormatTool;

impl FormatTool {
    /// Create a new `FormatTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FormatTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FormatTool {
    fn name(&self) -> &str {
        "format"
    }

    fn description(&self) -> &str {
        "Formats a string by substituting placeholders with provided values."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FormatTool is a stub");
        Ok("Result from format".into())
    }
}
