//! Base45 encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes the input string using Base45 encoding..
#[derive(Debug, Clone)]
pub struct EncodeBase45Tool;

impl EncodeBase45Tool {
    /// Create a new `EncodeBase45Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncodeBase45Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EncodeBase45Tool {
    fn name(&self) -> &str {
        "encode_base45"
    }

    fn description(&self) -> &str {
        "Encodes the input string using Base45 encoding."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EncodeBase45Tool is a stub");
        Ok("Result from encode_base45".into())
    }
}
