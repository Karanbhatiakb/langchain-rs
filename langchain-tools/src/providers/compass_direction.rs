//! Compass direction tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a bearing in degrees to a compass direction..
#[derive(Debug, Clone)]
pub struct CompassDirectionTool;

impl CompassDirectionTool {
    /// Create a new `CompassDirectionTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompassDirectionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CompassDirectionTool {
    fn name(&self) -> &str {
        "compass_direction"
    }

    fn description(&self) -> &str {
        "Converts a bearing in degrees to a compass direction."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CompassDirectionTool is a stub");
        Ok("Result from compass_direction".into())
    }
}
