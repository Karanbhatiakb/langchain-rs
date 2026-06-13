//! Multiplication math tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that multiplies two numbers.
#[derive(Debug, Clone)]
pub struct MultiplyTool;

impl MultiplyTool {
    /// Create a new `MultiplyTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MultiplyTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for MultiplyTool {
    fn name(&self) -> &str {
        "multiply"
    }

    fn description(&self) -> &str {
        "Multiplies two numbers. Input format: 'a b' returns a * b."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("MultiplyTool is a stub");
        Ok("Result from multiply".into())
    }
}
