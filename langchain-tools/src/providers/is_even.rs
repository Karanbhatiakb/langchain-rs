//! Even check tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks if a number is even.
#[derive(Debug, Clone)]
pub struct IsEvenTool;

impl IsEvenTool {
    /// Create a new `IsEvenTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for IsEvenTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for IsEvenTool {
    fn name(&self) -> &str {
        "is_even"
    }

    fn description(&self) -> &str {
        "Checks whether the input integer is even."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("IsEvenTool is a stub");
        Ok("Result from is_even".into())
    }
}
