//! Octal to decimal conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts an octal string to decimal.
#[derive(Debug, Clone)]
pub struct OctalToDecimalTool;

impl OctalToDecimalTool {
    /// Create a new `OctalToDecimalTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for OctalToDecimalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for OctalToDecimalTool {
    fn name(&self) -> &str {
        "octal_to_decimal"
    }

    fn description(&self) -> &str {
        "Converts an octal string to its decimal equivalent."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("OctalToDecimalTool is a stub");
        Ok("Result from octal_to_decimal".into())
    }
}
