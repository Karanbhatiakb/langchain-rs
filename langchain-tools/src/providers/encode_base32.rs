//! Base32 encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes the input string using Base32 encoding..
#[derive(Debug, Clone)]
pub struct EncodeBase32Tool;

impl EncodeBase32Tool {
    /// Create a new `EncodeBase32Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncodeBase32Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EncodeBase32Tool {
    fn name(&self) -> &str {
        "encode_base32"
    }

    fn description(&self) -> &str {
        "Encodes the input string using Base32 encoding."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EncodeBase32Tool is a stub");
        Ok("Result from encode_base32".into())
    }
}
