//! File delete tool implementation.
//!
//! Provides a `FileDeleteTool` that deletes a file from the file system.
//! Gated behind the `file_delete` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for deleting a file from the local file system.
#[derive(Debug, Clone)]
pub struct FileDeleteTool;

impl FileDeleteTool {
    /// Create a new `FileDeleteTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for FileDeleteTool {
    fn name(&self) -> &str {
        "file_delete"
    }

    fn description(&self) -> &str {
        "Delete a file from the local file system. \
         Input should be the path to the file to delete."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileDeleteTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
