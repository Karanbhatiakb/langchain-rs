//! Power/exponentiation math tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that raises a number to a power.
#[derive(Debug, Clone)]
pub struct PowerTool;

impl PowerTool {
    /// Create a new `PowerTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PowerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for PowerTool {
    fn name(&self) -> &str {
        "power"
    }

    fn description(&self) -> &str {
        "Raises a number to a power. Input format: 'base exp' returns base^exp."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("PowerTool is a stub");
        Ok("Result from power".into())
    }
}
