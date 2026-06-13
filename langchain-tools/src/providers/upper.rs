//! Uppercase string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a string to uppercase.
#[derive(Debug, Clone)]
pub struct UpperTool;

impl UpperTool {
    /// Create a new `UpperTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for UpperTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for UpperTool {
    fn name(&self) -> &str {
        "upper"
    }

    fn description(&self) -> &str {
        "Converts the input string to uppercase."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("UpperTool is a stub");
        Ok("Result from upper".into())
    }
}
