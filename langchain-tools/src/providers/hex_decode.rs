//! Hex decode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that decodes a hexadecimal string.
#[derive(Debug, Clone)]
pub struct HexDecodeTool;

impl HexDecodeTool {
    /// Create a new `HexDecodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HexDecodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for HexDecodeTool {
    fn name(&self) -> &str {
        "hex_decode"
    }

    fn description(&self) -> &str {
        "Decodes a hexadecimal string back to plain text."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HexDecodeTool is a stub");
        Ok("Result from hex_decode".into())
    }
}
