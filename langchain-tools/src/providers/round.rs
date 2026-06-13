//! Round tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that rounds a number to a given precision.
#[derive(Debug, Clone)]
pub struct RoundTool;

impl RoundTool {
    /// Create a new `RoundTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RoundTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for RoundTool {
    fn name(&self) -> &str {
        "round"
    }

    fn description(&self) -> &str {
        "Rounds the input number to the specified number of decimal places."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("RoundTool is a stub");
        Ok("Result from round".into())
    }
}
