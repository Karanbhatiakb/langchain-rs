//! Square root math tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the square root of a number.
#[derive(Debug, Clone)]
pub struct SqrtTool;

impl SqrtTool {
    /// Create a new `SqrtTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqrtTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for SqrtTool {
    fn name(&self) -> &str {
        "sqrt"
    }

    fn description(&self) -> &str {
        "Computes the square root of a number. Input: a single number."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SqrtTool is a stub");
        Ok("Result from sqrt".into())
    }
}
