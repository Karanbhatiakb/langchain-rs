//! Color HSL tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a color value to HSL components..
#[derive(Debug, Clone)]
pub struct ColorHslTool;

impl ColorHslTool {
    /// Create a new `ColorHslTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColorHslTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ColorHslTool {
    fn name(&self) -> &str {
        "color_hsl"
    }

    fn description(&self) -> &str {
        "Converts a color value to HSL components."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ColorHslTool is a stub");
        Ok("Result from color_hsl".into())
    }
}
