//! Bytes to megabytes conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts bytes to megabytes.
#[derive(Debug, Clone)]
pub struct BytesToMbTool;

impl BytesToMbTool {
    /// Create a new `BytesToMbTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BytesToMbTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BytesToMbTool {
    fn name(&self) -> &str {
        "bytes_to_mb"
    }

    fn description(&self) -> &str {
        "Converts a size in bytes to megabytes (1 MB = 1,048,576 bytes)."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BytesToMbTool is a stub");
        Ok("Result from bytes_to_mb".into())
    }
}
