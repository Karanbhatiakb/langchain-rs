//! Coordinates format tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that formats latitude and longitude coordinates..
#[derive(Debug, Clone)]
pub struct CoordinatesTool;

impl CoordinatesTool {
    /// Create a new `CoordinatesTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CoordinatesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CoordinatesTool {
    fn name(&self) -> &str {
        "coordinates"
    }

    fn description(&self) -> &str {
        "Formats latitude and longitude coordinates."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CoordinatesTool is a stub");
        Ok("Result from coordinates".into())
    }
}
