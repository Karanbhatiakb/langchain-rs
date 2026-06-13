//! File append tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that appends content to a file.
#[derive(Debug, Clone)]
pub struct FileAppendTool;

impl FileAppendTool {
    /// Create a new `FileAppendTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileAppendTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FileAppendTool {
    fn name(&self) -> &str {
        "file_append"
    }

    fn description(&self) -> &str {
        "Appends text content to the end of a file."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileAppendTool is a stub");
        Ok("Result from file_append".into())
    }
}
