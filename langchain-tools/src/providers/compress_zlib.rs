//! Zlib compress tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that compresses the input string using zlib..
#[derive(Debug, Clone)]
pub struct CompressZlibTool;

impl CompressZlibTool {
    /// Create a new `CompressZlibTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompressZlibTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CompressZlibTool {
    fn name(&self) -> &str {
        "compress_zlib"
    }

    fn description(&self) -> &str {
        "Compresses the input string using zlib."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CompressZlibTool is a stub");
        Ok("Result from compress_zlib".into())
    }
}
