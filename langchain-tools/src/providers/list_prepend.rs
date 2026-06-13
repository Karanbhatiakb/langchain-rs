//! List prepend tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that prepends a value to a list.
#[derive(Debug, Clone)]
pub struct ListPrependTool;

impl ListPrependTool {
    /// Create a new `ListPrependTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListPrependTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ListPrependTool {
    fn name(&self) -> &str {
        "list_prepend"
    }

    fn description(&self) -> &str {
        "Prepends a value to the beginning of a list."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ListPrependTool is a stub");
        Ok("Result from list_prepend".into())
    }
}
