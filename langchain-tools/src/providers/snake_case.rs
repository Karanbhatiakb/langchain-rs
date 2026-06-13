//! Snake case string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a string to snake_case.
#[derive(Debug, Clone)]
pub struct SnakeCaseTool;

impl SnakeCaseTool {
    /// Create a new `SnakeCaseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SnakeCaseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for SnakeCaseTool {
    fn name(&self) -> &str {
        "snake_case"
    }

    fn description(&self) -> &str {
        "Converts the input string to snake_case."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SnakeCaseTool is a stub");
        Ok("Result from snake_case".into())
    }
}
