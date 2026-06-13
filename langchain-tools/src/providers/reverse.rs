//! Reverse string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that reverses a string.
#[derive(Debug, Clone)]
pub struct ReverseTool;

impl ReverseTool {
    /// Create a new `ReverseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReverseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ReverseTool {
    fn name(&self) -> &str {
        "reverse"
    }

    fn description(&self) -> &str {
        "Reverses the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ReverseTool is a stub");
        Ok("Result from reverse".into())
    }
}
