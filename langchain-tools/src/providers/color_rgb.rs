//! Color RGB tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a color value to RGB components..
#[derive(Debug, Clone)]
pub struct ColorRgbTool;

impl ColorRgbTool {
    /// Create a new `ColorRgbTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColorRgbTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ColorRgbTool {
    fn name(&self) -> &str {
        "color_rgb"
    }

    fn description(&self) -> &str {
        "Converts a color value to RGB components."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ColorRgbTool is a stub");
        Ok("Result from color_rgb".into())
    }
}
