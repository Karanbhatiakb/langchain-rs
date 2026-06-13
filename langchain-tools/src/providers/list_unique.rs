//! List unique tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns unique elements from a list.
#[derive(Debug, Clone)]
pub struct ListUniqueTool;

impl ListUniqueTool {
    /// Create a new `ListUniqueTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListUniqueTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ListUniqueTool {
    fn name(&self) -> &str {
        "list_unique"
    }

    fn description(&self) -> &str {
        "Returns the unique elements from a list, removing duplicates."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ListUniqueTool is a stub");
        Ok("Result from list_unique".into())
    }
}
