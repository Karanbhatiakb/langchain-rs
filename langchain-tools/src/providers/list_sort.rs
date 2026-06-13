//! List sort tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that sorts a list.
#[derive(Debug, Clone)]
pub struct ListSortTool;

impl ListSortTool {
    /// Create a new `ListSortTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListSortTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ListSortTool {
    fn name(&self) -> &str {
        "list_sort"
    }

    fn description(&self) -> &str {
        "Sorts the elements of a list in ascending order."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ListSortTool is a stub");
        Ok("Result from list_sort".into())
    }
}
