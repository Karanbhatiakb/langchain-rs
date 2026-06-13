//! Floor tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the floor of a number.
#[derive(Debug, Clone)]
pub struct FloorTool;

impl FloorTool {
    /// Create a new `FloorTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FloorTool {
    fn name(&self) -> &str {
        "floor"
    }

    fn description(&self) -> &str {
        "Returns the largest integer less than or equal to the input number."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FloorTool is a stub");
        Ok("Result from floor".into())
    }
}
