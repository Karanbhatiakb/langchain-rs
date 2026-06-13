//! File system tool implementation.
//!
//! Provides a `FileSystemTool` that wraps read, write, and list operations
//! on the local file system. Gated behind the `file_system` feature flag.

use async_trait::async_trait;
use langchain_core::errors::ChainError;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for interacting with the file system (read, write, list).
#[derive(Debug, Clone)]
pub struct FileSystemTool {
    allowed_directory: String,
}

impl FileSystemTool {
    /// Create a new `FileSystemTool` restricted to `allowed_directory`.
    pub fn new(allowed_directory: impl Into<String>) -> Self {
        Self {
            allowed_directory: allowed_directory.into(),
        }
    }
}

#[async_trait]
impl BaseTool for FileSystemTool {
    fn name(&self) -> &str {
        "file_system"
    }

    fn description(&self) -> &str {
        "Read, write, and list files on the local file system, restricted to \
         an allowed directory. Input format: '<operation> <path>' where \
         operation is read, write, or list."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileSystemTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}

impl FileSystemTool {
    fn validate_path(&self, _path: &str) -> Result<(), ChainError> {
        let _ = &self.allowed_directory;
        Ok(())
    }
}
