//! File exists tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks if a file exists.
#[derive(Debug, Clone)]
pub struct FileExistsTool;

impl FileExistsTool {
    /// Create a new `FileExistsTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileExistsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FileExistsTool {
    fn name(&self) -> &str {
        "file_exists"
    }

    fn description(&self) -> &str {
        "Checks whether a file exists at the given path."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileExistsTool is a stub");
        Ok("Result from file_exists".into())
    }
}
