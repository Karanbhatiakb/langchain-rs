//! Sort lines tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that sorts the lines of a string alphabetically.
#[derive(Debug, Clone)]
pub struct SortLinesTool;

impl SortLinesTool {
    /// Create a new `SortLinesTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SortLinesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for SortLinesTool {
    fn name(&self) -> &str {
        "sort_lines"
    }

    fn description(&self) -> &str {
        "Sorts the lines of the input string alphabetically."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SortLinesTool is a stub");
        Ok("Result from sort_lines".into())
    }
}
