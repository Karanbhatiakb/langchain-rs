//! Subtraction math tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that subtracts two numbers.
#[derive(Debug, Clone)]
pub struct SubtractTool;

impl SubtractTool {
    /// Create a new `SubtractTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SubtractTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for SubtractTool {
    fn name(&self) -> &str {
        "subtract"
    }

    fn description(&self) -> &str {
        "Subtracts two numbers. Input format: 'a b' returns a - b."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SubtractTool is a stub");
        Ok("Result from subtract".into())
    }
}
