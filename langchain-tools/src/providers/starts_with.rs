//! Starts-with check tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks if a string starts with a given prefix.
#[derive(Debug, Clone)]
pub struct StartsWithTool;

impl StartsWithTool {
    /// Create a new `StartsWithTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for StartsWithTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for StartsWithTool {
    fn name(&self) -> &str {
        "starts_with"
    }

    fn description(&self) -> &str {
        "Checks whether the input string starts with a given prefix."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("StartsWithTool is a stub");
        Ok("Result from starts_with".into())
    }
}
