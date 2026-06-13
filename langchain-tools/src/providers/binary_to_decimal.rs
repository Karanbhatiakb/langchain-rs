//! Binary to decimal conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a binary string to decimal.
#[derive(Debug, Clone)]
pub struct BinaryToDecimalTool;

impl BinaryToDecimalTool {
    /// Create a new `BinaryToDecimalTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BinaryToDecimalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BinaryToDecimalTool {
    fn name(&self) -> &str {
        "binary_to_decimal"
    }

    fn description(&self) -> &str {
        "Converts a binary string to its decimal equivalent."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BinaryToDecimalTool is a stub");
        Ok("Result from binary_to_decimal".into())
    }
}
