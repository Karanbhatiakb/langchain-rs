//! File search tool implementation.
//!
//! Provides a `FileSearchTool` that searches for files by glob pattern.
//! Gated behind the `file_search` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for searching files by glob pattern.
#[derive(Debug, Clone)]
pub struct FileSearchTool;

impl FileSearchTool {
    /// Create a new `FileSearchTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for FileSearchTool {
    fn name(&self) -> &str {
        "file_search"
    }

    fn description(&self) -> &str {
        "Search for files on the local file system using a glob pattern. \
         Input should be a glob pattern (e.g. '**/*.rs')."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FileSearchTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
