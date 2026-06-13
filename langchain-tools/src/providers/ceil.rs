//! Ceil tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the ceiling of a number.
#[derive(Debug, Clone)]
pub struct CeilTool;

impl CeilTool {
    /// Create a new `CeilTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CeilTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CeilTool {
    fn name(&self) -> &str {
        "ceil"
    }

    fn description(&self) -> &str {
        "Returns the smallest integer greater than or equal to the input number."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CeilTool is a stub");
        Ok("Result from ceil".into())
    }
}
