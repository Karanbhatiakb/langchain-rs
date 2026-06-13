//! Decimal to binary conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a decimal number to binary.
#[derive(Debug, Clone)]
pub struct DecimalToBinaryTool;

impl DecimalToBinaryTool {
    /// Create a new `DecimalToBinaryTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DecimalToBinaryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DecimalToBinaryTool {
    fn name(&self) -> &str {
        "decimal_to_binary"
    }

    fn description(&self) -> &str {
        "Converts a decimal number to its binary representation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DecimalToBinaryTool is a stub");
        Ok("Result from decimal_to_binary".into())
    }
}
