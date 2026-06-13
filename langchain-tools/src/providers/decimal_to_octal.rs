//! Decimal to octal conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a decimal number to octal.
#[derive(Debug, Clone)]
pub struct DecimalToOctalTool;

impl DecimalToOctalTool {
    /// Create a new `DecimalToOctalTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DecimalToOctalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DecimalToOctalTool {
    fn name(&self) -> &str {
        "decimal_to_octal"
    }

    fn description(&self) -> &str {
        "Converts a decimal number to its octal representation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DecimalToOctalTool is a stub");
        Ok("Result from decimal_to_octal".into())
    }
}
