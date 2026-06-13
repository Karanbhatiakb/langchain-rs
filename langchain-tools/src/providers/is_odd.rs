//! Odd check tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks if a number is odd.
#[derive(Debug, Clone)]
pub struct IsOddTool;

impl IsOddTool {
    /// Create a new `IsOddTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for IsOddTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for IsOddTool {
    fn name(&self) -> &str {
        "is_odd"
    }

    fn description(&self) -> &str {
        "Checks whether the input integer is odd."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("IsOddTool is a stub");
        Ok("Result from is_odd".into())
    }
}
