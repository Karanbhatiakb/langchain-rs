//! Haversine distance tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that calculates the great-circle distance between two coordinates on a sphere..
#[derive(Debug, Clone)]
pub struct DistanceHaversineTool;

impl DistanceHaversineTool {
    /// Create a new `DistanceHaversineTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DistanceHaversineTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DistanceHaversineTool {
    fn name(&self) -> &str {
        "distance_haversine"
    }

    fn description(&self) -> &str {
        "Calculates the great-circle distance between two coordinates on a sphere."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DistanceHaversineTool is a stub");
        Ok("Result from distance_haversine".into())
    }
}
