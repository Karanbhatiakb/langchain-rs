//! String split tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that splits a string by a delimiter.
#[derive(Debug, Clone)]
pub struct SplitTool;

impl SplitTool {
    /// Create a new `SplitTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SplitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for SplitTool {
    fn name(&self) -> &str {
        "split"
    }

    fn description(&self) -> &str {
        "Splits the input string by a given delimiter."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SplitTool is a stub");
        Ok("Result from split".into())
    }
}
