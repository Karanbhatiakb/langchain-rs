//! Gzip compress tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that compresses the input string using gzip..
#[derive(Debug, Clone)]
pub struct CompressGzipTool;

impl CompressGzipTool {
    /// Create a new `CompressGzipTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompressGzipTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CompressGzipTool {
    fn name(&self) -> &str {
        "compress_gzip"
    }

    fn description(&self) -> &str {
        "Compresses the input string using gzip."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CompressGzipTool is a stub");
        Ok("Result from compress_gzip".into())
    }
}
