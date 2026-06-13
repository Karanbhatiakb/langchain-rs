//! File read tool implementation.
//!
//! Provides a `FileReadTool` that reads the contents of a file.
//! Gated behind the `file_read` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for reading file contents from the local file system.
#[derive(Debug, Clone)]
pub struct FileReadTool;

impl FileReadTool {
    /// Create a new `FileReadTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file on the local file system. \
         Input should be an absolute or relative file path."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileReadTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
