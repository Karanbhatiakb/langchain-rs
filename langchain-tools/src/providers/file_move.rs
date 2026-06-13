//! File move tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that moves or renames a file.
#[derive(Debug, Clone)]
pub struct FileMoveTool;

impl FileMoveTool {
    /// Create a new `FileMoveTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileMoveTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FileMoveTool {
    fn name(&self) -> &str {
        "file_move"
    }

    fn description(&self) -> &str {
        "Moves or renames a file from source path to destination path."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileMoveTool is a stub");
        Ok("Result from file_move".into())
    }
}
