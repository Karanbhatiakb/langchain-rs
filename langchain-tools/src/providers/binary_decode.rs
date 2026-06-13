//! Binary decode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that decodes a binary string.
#[derive(Debug, Clone)]
pub struct BinaryDecodeTool;

impl BinaryDecodeTool {
    /// Create a new `BinaryDecodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BinaryDecodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BinaryDecodeTool {
    fn name(&self) -> &str {
        "binary_decode"
    }

    fn description(&self) -> &str {
        "Decodes a binary (base-2) string back to plain text."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BinaryDecodeTool is a stub");
        Ok("Result from binary_decode".into())
    }
}
