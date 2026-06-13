//! Least common multiple tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the least common multiple of two numbers.
#[derive(Debug, Clone)]
pub struct LcmTool;

impl LcmTool {
    /// Create a new `LcmTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LcmTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for LcmTool {
    fn name(&self) -> &str {
        "lcm"
    }

    fn description(&self) -> &str {
        "Computes the least common multiple of two integers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("LcmTool is a stub");
        Ok("Result from lcm".into())
    }
}
