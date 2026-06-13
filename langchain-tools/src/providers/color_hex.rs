//! Color hex tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a color value to hexadecimal format..
#[derive(Debug, Clone)]
pub struct ColorHexTool;

impl ColorHexTool {
    /// Create a new `ColorHexTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColorHexTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ColorHexTool {
    fn name(&self) -> &str {
        "color_hex"
    }

    fn description(&self) -> &str {
        "Converts a color value to hexadecimal format."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ColorHexTool is a stub");
        Ok("Result from color_hex".into())
    }
}
