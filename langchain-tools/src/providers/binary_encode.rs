//! Binary encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes a string as binary.
#[derive(Debug, Clone)]
pub struct BinaryEncodeTool;

impl BinaryEncodeTool {
    /// Create a new `BinaryEncodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BinaryEncodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BinaryEncodeTool {
    fn name(&self) -> &str {
        "binary_encode"
    }

    fn description(&self) -> &str {
        "Encodes the input string as binary (base-2)."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BinaryEncodeTool is a stub");
        Ok("Result from binary_encode".into())
    }
}
