//! Ends-with check tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks if a string ends with a given suffix.
#[derive(Debug, Clone)]
pub struct EndsWithTool;

impl EndsWithTool {
    /// Create a new `EndsWithTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for EndsWithTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for EndsWithTool {
    fn name(&self) -> &str {
        "ends_with"
    }

    fn description(&self) -> &str {
        "Checks whether the input string ends with a given suffix."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EndsWithTool is a stub");
        Ok("Result from ends_with".into())
    }
}
