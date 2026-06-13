//! Color name tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that looks up the name of a color from its hex or RGB value..
#[derive(Debug, Clone)]
pub struct ColorNameTool;

impl ColorNameTool {
    /// Create a new `ColorNameTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColorNameTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ColorNameTool {
    fn name(&self) -> &str {
        "color_name"
    }

    fn description(&self) -> &str {
        "Looks up the name of a color from its hex or RGB value."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ColorNameTool is a stub");
        Ok("Result from color_name".into())
    }
}
