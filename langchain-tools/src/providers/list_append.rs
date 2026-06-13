//! List append tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that appends a value to a list.
#[derive(Debug, Clone)]
pub struct ListAppendTool;

impl ListAppendTool {
    /// Create a new `ListAppendTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListAppendTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ListAppendTool {
    fn name(&self) -> &str {
        "list_append"
    }

    fn description(&self) -> &str {
        "Appends a value to the end of a list."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ListAppendTool is a stub");
        Ok("Result from list_append".into())
    }
}
