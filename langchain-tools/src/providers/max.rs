//! Maximum tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the maximum of a set of numbers.
#[derive(Debug, Clone)]
pub struct MaxTool;

impl MaxTool {
    /// Create a new `MaxTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MaxTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for MaxTool {
    fn name(&self) -> &str {
        "max"
    }

    fn description(&self) -> &str {
        "Returns the maximum value from a list of numbers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("MaxTool is a stub");
        Ok("Result from max".into())
    }
}
