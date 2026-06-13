//! Brightness adjust tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that adjusts the screen brightness to the given level..
#[derive(Debug, Clone)]
pub struct BrightnessTool;

impl BrightnessTool {
    /// Create a new `BrightnessTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BrightnessTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BrightnessTool {
    fn name(&self) -> &str {
        "brightness"
    }

    fn description(&self) -> &str {
        "Adjusts the screen brightness to the given level."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BrightnessTool is a stub");
        Ok("Result from brightness".into())
    }
}
