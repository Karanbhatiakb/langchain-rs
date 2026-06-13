//! Factorial tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the factorial of a non-negative integer.
#[derive(Debug, Clone)]
pub struct FactorialTool;

impl FactorialTool {
    /// Create a new `FactorialTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FactorialTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FactorialTool {
    fn name(&self) -> &str {
        "factorial"
    }

    fn description(&self) -> &str {
        "Computes the factorial of a non-negative integer."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FactorialTool is a stub");
        Ok("Result from factorial".into())
    }
}
