//! Pad string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that pads a string to a specified width.
#[derive(Debug, Clone)]
pub struct PadTool;

impl PadTool {
    /// Create a new `PadTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for PadTool {
    fn name(&self) -> &str {
        "pad"
    }

    fn description(&self) -> &str {
        "Pads the input string to a specified width. Input format: 'width:char:text'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("PadTool is a stub");
        Ok("Result from pad".into())
    }
}
