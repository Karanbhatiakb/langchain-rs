//! Trim whitespace tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that trims leading and trailing whitespace from a string.
#[derive(Debug, Clone)]
pub struct TrimTool;

impl TrimTool {
    /// Create a new `TrimTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TrimTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for TrimTool {
    fn name(&self) -> &str {
        "trim"
    }

    fn description(&self) -> &str {
        "Trims leading and trailing whitespace from the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("TrimTool is a stub");
        Ok("Result from trim".into())
    }
}
