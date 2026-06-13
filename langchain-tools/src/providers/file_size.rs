//! File size tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the size of a file.
#[derive(Debug, Clone)]
pub struct FileSizeTool;

impl FileSizeTool {
    /// Create a new `FileSizeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileSizeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FileSizeTool {
    fn name(&self) -> &str {
        "file_size"
    }

    fn description(&self) -> &str {
        "Returns the size of a file in bytes."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileSizeTool is a stub");
        Ok("Result from file_size".into())
    }
}
