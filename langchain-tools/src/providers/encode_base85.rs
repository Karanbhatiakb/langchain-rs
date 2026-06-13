//! Base85 encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes the input string using Base85 encoding..
#[derive(Debug, Clone)]
pub struct EncodeBase85Tool;

impl EncodeBase85Tool {
    /// Create a new `EncodeBase85Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncodeBase85Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EncodeBase85Tool {
    fn name(&self) -> &str {
        "encode_base85"
    }

    fn description(&self) -> &str {
        "Encodes the input string using Base85 encoding."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EncodeBase85Tool is a stub");
        Ok("Result from encode_base85".into())
    }
}
