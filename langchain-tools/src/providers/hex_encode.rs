//! Hex encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes a string as hexadecimal.
#[derive(Debug, Clone)]
pub struct HexEncodeTool;

impl HexEncodeTool {
    /// Create a new `HexEncodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HexEncodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for HexEncodeTool {
    fn name(&self) -> &str {
        "hex_encode"
    }

    fn description(&self) -> &str {
        "Encodes the input string as hexadecimal."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HexEncodeTool is a stub");
        Ok("Result from hex_encode".into())
    }
}
