//! Zstd compress tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that compresses the input string using Zstandard..
#[derive(Debug, Clone)]
pub struct CompressZstdTool;

impl CompressZstdTool {
    /// Create a new `CompressZstdTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompressZstdTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CompressZstdTool {
    fn name(&self) -> &str {
        "compress_zstd"
    }

    fn description(&self) -> &str {
        "Compresses the input string using Zstandard."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CompressZstdTool is a stub");
        Ok("Result from compress_zstd".into())
    }
}
