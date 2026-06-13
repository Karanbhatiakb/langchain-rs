//! File rename tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that renames a file.
#[derive(Debug, Clone)]
pub struct FileRenameTool;

impl FileRenameTool {
    /// Create a new `FileRenameTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileRenameTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FileRenameTool {
    fn name(&self) -> &str {
        "file_rename"
    }

    fn description(&self) -> &str {
        "Renames a file from the old name to the new name."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileRenameTool is a stub");
        Ok("Result from file_rename".into())
    }
}
