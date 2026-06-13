//! Color code tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a color name to its numeric code representation..
#[derive(Debug, Clone)]
pub struct ColorCodeTool;

impl ColorCodeTool {
    /// Create a new `ColorCodeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColorCodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ColorCodeTool {
    fn name(&self) -> &str {
        "color_code"
    }

    fn description(&self) -> &str {
        "Converts a color name to its numeric code representation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ColorCodeTool is a stub");
        Ok("Result from color_code".into())
    }
}
