//! Base62 encode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes the input string using Base62 encoding..
#[derive(Debug, Clone)]
pub struct EncodeBase62Tool;

impl EncodeBase62Tool {
    /// Create a new `EncodeBase62Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncodeBase62Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EncodeBase62Tool {
    fn name(&self) -> &str {
        "encode_base62"
    }

    fn description(&self) -> &str {
        "Encodes the input string using Base62 encoding."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EncodeBase62Tool is a stub");
        Ok("Result from encode_base62".into())
    }
}
