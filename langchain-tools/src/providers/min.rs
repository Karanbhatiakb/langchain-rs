//! Minimum tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the minimum of a set of numbers.
#[derive(Debug, Clone)]
pub struct MinTool;

impl MinTool {
    /// Create a new `MinTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MinTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for MinTool {
    fn name(&self) -> &str {
        "min"
    }

    fn description(&self) -> &str {
        "Returns the minimum value from a list of numbers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("MinTool is a stub");
        Ok("Result from min".into())
    }
}
