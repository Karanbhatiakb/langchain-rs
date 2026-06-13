//! Battery status tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the current battery status of the device..
#[derive(Debug, Clone)]
pub struct BatteryTool;

impl BatteryTool {
    /// Create a new `BatteryTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BatteryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BatteryTool {
    fn name(&self) -> &str {
        "battery"
    }

    fn description(&self) -> &str {
        "Returns the current battery status of the device."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BatteryTool is a stub");
        Ok("Result from battery".into())
    }
}
