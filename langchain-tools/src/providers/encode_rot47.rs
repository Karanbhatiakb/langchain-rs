//! ROT47 tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that applies the ROT47 cipher to the input string..
#[derive(Debug, Clone)]
pub struct EncodeRot47Tool;

impl EncodeRot47Tool {
    /// Create a new `EncodeRot47Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncodeRot47Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EncodeRot47Tool {
    fn name(&self) -> &str {
        "encode_rot47"
    }

    fn description(&self) -> &str {
        "Applies the ROT47 cipher to the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EncodeRot47Tool is a stub");
        Ok("Result from encode_rot47".into())
    }
}
