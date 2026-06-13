//! Lowercase string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a string to lowercase.
#[derive(Debug, Clone)]
pub struct LowerTool;

impl LowerTool {
    /// Create a new `LowerTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LowerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for LowerTool {
    fn name(&self) -> &str {
        "lower"
    }

    fn description(&self) -> &str {
        "Converts the input string to lowercase."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("LowerTool is a stub");
        Ok("Result from lower".into())
    }
}
