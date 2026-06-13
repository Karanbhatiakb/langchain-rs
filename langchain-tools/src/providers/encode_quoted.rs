//! Quoted printable tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that encodes input as quoted-printable text..
#[derive(Debug, Clone)]
pub struct EncodeQuotedTool;

impl EncodeQuotedTool {
    /// Create a new `EncodeQuotedTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EncodeQuotedTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EncodeQuotedTool {
    fn name(&self) -> &str {
        "encode_quoted"
    }

    fn description(&self) -> &str {
        "Encodes input as quoted-printable text."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EncodeQuotedTool is a stub");
        Ok("Result from encode_quoted".into())
    }
}
