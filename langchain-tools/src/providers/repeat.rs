//! String repeat tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that repeats a string a given number of times.
#[derive(Debug, Clone)]
pub struct RepeatTool;

impl RepeatTool {
    /// Create a new `RepeatTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RepeatTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for RepeatTool {
    fn name(&self) -> &str {
        "repeat"
    }

    fn description(&self) -> &str {
        "Repeats the input string a specified number of times."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("RepeatTool is a stub");
        Ok("Result from repeat".into())
    }
}
