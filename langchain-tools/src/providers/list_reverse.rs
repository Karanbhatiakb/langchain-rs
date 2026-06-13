//! List reverse tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that reverses the order of elements in a list.
#[derive(Debug, Clone)]
pub struct ListReverseTool;

impl ListReverseTool {
    /// Create a new `ListReverseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListReverseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ListReverseTool {
    fn name(&self) -> &str {
        "list_reverse"
    }

    fn description(&self) -> &str {
        "Reverses the order of elements in a list."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ListReverseTool is a stub");
        Ok("Result from list_reverse".into())
    }
}
