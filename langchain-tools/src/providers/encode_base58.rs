//! Base58 encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes the input string using Base58 encoding..
#[derive(Debug, Clone)]
pub struct EncodeBase58Tool;

impl EncodeBase58Tool {
    /// Create a new `EncodeBase58Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncodeBase58Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EncodeBase58Tool {
    fn name(&self) -> &str {
        "encode_base58"
    }

    fn description(&self) -> &str {
        "Encodes the input string using Base58 encoding."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EncodeBase58Tool is a stub");
        Ok("Result from encode_base58".into())
    }
}
