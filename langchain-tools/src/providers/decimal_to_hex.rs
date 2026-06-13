//! Decimal to hex conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a decimal number to hexadecimal.
#[derive(Debug, Clone)]
pub struct DecimalToHexTool;

impl DecimalToHexTool {
    /// Create a new `DecimalToHexTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DecimalToHexTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DecimalToHexTool {
    fn name(&self) -> &str {
        "decimal_to_hex"
    }

    fn description(&self) -> &str {
        "Converts a decimal number to its hexadecimal representation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DecimalToHexTool is a stub");
        Ok("Result from decimal_to_hex".into())
    }
}
