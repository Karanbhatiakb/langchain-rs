//! LZ4 compress tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that compresses the input string using LZ4..
#[derive(Debug, Clone)]
pub struct CompressLz4Tool;

impl CompressLz4Tool {
    /// Create a new `CompressLz4Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompressLz4Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CompressLz4Tool {
    fn name(&self) -> &str {
        "compress_lz4"
    }

    fn description(&self) -> &str {
        "Compresses the input string using LZ4."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CompressLz4Tool is a stub");
        Ok("Result from compress_lz4".into())
    }
}
