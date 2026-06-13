//! String length tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the length of a string.
#[derive(Debug, Clone)]
pub struct LengthTool;

impl LengthTool {
    /// Create a new `LengthTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LengthTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for LengthTool {
    fn name(&self) -> &str {
        "length"
    }

    fn description(&self) -> &str {
        "Returns the length of the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("LengthTool is a stub");
        Ok("Result from length".into())
    }
}
