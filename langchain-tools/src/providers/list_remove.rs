//! List remove tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that removes a value from a list.
#[derive(Debug, Clone)]
pub struct ListRemoveTool;

impl ListRemoveTool {
    /// Create a new `ListRemoveTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListRemoveTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ListRemoveTool {
    fn name(&self) -> &str {
        "list_remove"
    }

    fn description(&self) -> &str {
        "Removes a value from a list by index or value."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ListRemoveTool is a stub");
        Ok("Result from list_remove".into())
    }
}
