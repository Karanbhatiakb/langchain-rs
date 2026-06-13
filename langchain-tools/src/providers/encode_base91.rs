//! Base91 encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes the input string using Base91 encoding..
#[derive(Debug, Clone)]
pub struct EncodeBase91Tool;

impl EncodeBase91Tool {
    /// Create a new `EncodeBase91Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncodeBase91Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EncodeBase91Tool {
    fn name(&self) -> &str {
        "encode_base91"
    }

    fn description(&self) -> &str {
        "Encodes the input string using Base91 encoding."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EncodeBase91Tool is a stub");
        Ok("Result from encode_base91".into())
    }
}
