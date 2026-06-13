//! Manhattan distance tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that calculates the Manhattan distance between two points..
#[derive(Debug, Clone)]
pub struct DistanceManhattanTool;

impl DistanceManhattanTool {
    /// Create a new `DistanceManhattanTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DistanceManhattanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DistanceManhattanTool {
    fn name(&self) -> &str {
        "distance_manhattan"
    }

    fn description(&self) -> &str {
        "Calculates the Manhattan distance between two points."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DistanceManhattanTool is a stub");
        Ok("Result from distance_manhattan".into())
    }
}
