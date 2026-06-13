//! Directory create tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that creates a directory.
#[derive(Debug, Clone)]
pub struct DirCreateTool;

impl DirCreateTool {
    /// Create a new `DirCreateTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DirCreateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DirCreateTool {
    fn name(&self) -> &str {
        "dir_create"
    }

    fn description(&self) -> &str {
        "Creates a directory at the given path."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DirCreateTool is a stub");
        Ok("Result from dir_create".into())
    }
}
