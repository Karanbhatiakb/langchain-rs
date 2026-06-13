//! Capitalize string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that capitalizes the first character of a string.
#[derive(Debug, Clone)]
pub struct CapitalizeTool;

impl CapitalizeTool {
    /// Create a new `CapitalizeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CapitalizeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CapitalizeTool {
    fn name(&self) -> &str {
        "capitalize"
    }

    fn description(&self) -> &str {
        "Capitalizes the first character of the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CapitalizeTool is a stub");
        Ok("Result from capitalize".into())
    }
}
