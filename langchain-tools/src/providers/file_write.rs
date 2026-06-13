//! File write tool implementation.
//!
//! Provides a `FileWriteTool` that writes content to a file.
//! Gated behind the `file_write` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for writing content to a file on the local file system.
#[derive(Debug, Clone)]
pub struct FileWriteTool;

impl FileWriteTool {
    /// Create a new `FileWriteTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file on the local file system. \
         Input should be '<file_path>\\n<content>'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileWriteTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
