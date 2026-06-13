//! Meters to feet conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts metres to feet.
#[derive(Debug, Clone)]
pub struct MetersToFeetTool;

impl MetersToFeetTool {
    /// Create a new `MetersToFeetTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MetersToFeetTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for MetersToFeetTool {
    fn name(&self) -> &str {
        "meters_to_feet"
    }

    fn description(&self) -> &str {
        "Converts a length from metres to feet."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("MetersToFeetTool is a stub");
        Ok("Result from meters_to_feet".into())
    }
}
