//! File copy tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that copies a file from source to destination.
#[derive(Debug, Clone)]
pub struct FileCopyTool;

impl FileCopyTool {
    /// Create a new `FileCopyTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileCopyTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FileCopyTool {
    fn name(&self) -> &str {
        "file_copy"
    }

    fn description(&self) -> &str {
        "Copies a file from a source path to a destination path."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileCopyTool is a stub");
        Ok("Result from file_copy".into())
    }
}
