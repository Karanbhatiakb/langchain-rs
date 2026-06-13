//! Litres to gallons conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts litres to US gallons.
#[derive(Debug, Clone)]
pub struct LitersToGallonsTool;

impl LitersToGallonsTool {
    /// Create a new `LitersToGallonsTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LitersToGallonsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for LitersToGallonsTool {
    fn name(&self) -> &str {
        "liters_to_gallons"
    }

    fn description(&self) -> &str {
        "Converts a volume from litres to US gallons."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("LitersToGallonsTool is a stub");
        Ok("Result from liters_to_gallons".into())
    }
}
