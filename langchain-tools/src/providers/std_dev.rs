//! Standard deviation tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the standard deviation of a list of numbers.
#[derive(Debug, Clone)]
pub struct StdDevTool;

impl StdDevTool {
    /// Create a new `StdDevTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdDevTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for StdDevTool {
    fn name(&self) -> &str {
        "std_dev"
    }

    fn description(&self) -> &str {
        "Computes the standard deviation of a list of numbers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("StdDevTool is a stub");
        Ok("Result from std_dev".into())
    }
}
