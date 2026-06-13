//! Addition math tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that adds two numbers.
#[derive(Debug, Clone)]
pub struct AddTool;

impl AddTool {
    /// Create a new `AddTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AddTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AddTool {
    fn name(&self) -> &str {
        "add"
    }

    fn description(&self) -> &str {
        "Adds two numbers. Input format: 'a b' where a and b are numbers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("AddTool is a stub");
        Ok("Result from add".into())
    }
}
