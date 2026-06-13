//! Brotli compress tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that compresses the input string using Brotli..
#[derive(Debug, Clone)]
pub struct CompressBrotliTool;

impl CompressBrotliTool {
    /// Create a new `CompressBrotliTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompressBrotliTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CompressBrotliTool {
    fn name(&self) -> &str {
        "compress_brotli"
    }

    fn description(&self) -> &str {
        "Compresses the input string using Brotli."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CompressBrotliTool is a stub");
        Ok("Result from compress_brotli".into())
    }
}
