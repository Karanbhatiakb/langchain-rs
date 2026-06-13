//! String strip tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that strips leading and trailing characters from a string.
#[derive(Debug, Clone)]
pub struct StripTool;

impl StripTool {
    /// Create a new `StripTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for StripTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for StripTool {
    fn name(&self) -> &str {
        "strip"
    }

    fn description(&self) -> &str {
        "Strips leading and trailing whitespace (or given characters) from the input."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("StripTool is a stub");
        Ok("Result from strip".into())
    }
}
