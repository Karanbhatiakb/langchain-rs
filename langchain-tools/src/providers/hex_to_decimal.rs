//! Hex to decimal conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a hexadecimal string to decimal.
#[derive(Debug, Clone)]
pub struct HexToDecimalTool;

impl HexToDecimalTool {
    /// Create a new `HexToDecimalTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HexToDecimalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for HexToDecimalTool {
    fn name(&self) -> &str {
        "hex_to_decimal"
    }

    fn description(&self) -> &str {
        "Converts a hexadecimal string to its decimal equivalent."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HexToDecimalTool is a stub");
        Ok("Result from hex_to_decimal".into())
    }
}
