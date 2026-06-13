//! Directory list tool implementation.
//!
//! Provides a `DirectoryListTool` that lists the contents of a directory.
//! Gated behind the `directory_list` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for listing the contents of a directory.
#[derive(Debug, Clone)]
pub struct DirectoryListTool;

impl DirectoryListTool {
    /// Create a new `DirectoryListTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for DirectoryListTool {
    fn name(&self) -> &str {
        "directory_list"
    }

    fn description(&self) -> &str {
        "List the contents of a directory on the local file system. \
         Input should be a directory path."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DirectoryListTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
