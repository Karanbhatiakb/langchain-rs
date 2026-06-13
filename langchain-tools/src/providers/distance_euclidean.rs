//! Euclidean distance tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that calculates the Euclidean distance between two points in N-dimensional space..
#[derive(Debug, Clone)]
pub struct DistanceEuclideanTool;

impl DistanceEuclideanTool {
    /// Create a new `DistanceEuclideanTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DistanceEuclideanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DistanceEuclideanTool {
    fn name(&self) -> &str {
        "distance_euclidean"
    }

    fn description(&self) -> &str {
        "Calculates the Euclidean distance between two points in N-dimensional space."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DistanceEuclideanTool is a stub");
        Ok("Result from distance_euclidean".into())
    }
}
