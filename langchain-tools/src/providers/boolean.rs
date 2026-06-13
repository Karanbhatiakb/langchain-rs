//! Boolean parser tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that parses a boolean value from a string input..
#[derive(Debug, Clone)]
pub struct BooleanTool;

impl BooleanTool {
    /// Create a new `BooleanTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BooleanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BooleanTool {
    fn name(&self) -> &str {
        "boolean"
    }

    fn description(&self) -> &str {
        "Parses a boolean value from a string input."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BooleanTool is a stub");
        Ok("Result from boolean".into())
    }
}
