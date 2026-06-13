//! 3D distance tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that calculates the Euclidean distance between two 3D points..
#[derive(Debug, Clone)]
pub struct Distance3dTool;

impl Distance3dTool {
    /// Create a new `Distance3dTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Distance3dTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for Distance3dTool {
    fn name(&self) -> &str {
        "distance_3d"
    }

    fn description(&self) -> &str {
        "Calculates the Euclidean distance between two 3D points."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("Distance3dTool is a stub");
        Ok("Result from distance_3d".into())
    }
}
