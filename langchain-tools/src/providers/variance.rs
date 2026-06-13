//! Statistical variance tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the statistical variance of a list of numbers.
#[derive(Debug, Clone)]
pub struct VarianceTool;

impl VarianceTool {
    /// Create a new `VarianceTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for VarianceTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for VarianceTool {
    fn name(&self) -> &str {
        "variance"
    }

    fn description(&self) -> &str {
        "Computes the statistical variance of a list of numbers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("VarianceTool is a stub");
        Ok("Result from variance".into())
    }
}
