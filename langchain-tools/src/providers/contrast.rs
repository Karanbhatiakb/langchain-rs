//! Contrast tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the contrast ratio between two luminance values..
#[derive(Debug, Clone)]
pub struct ContrastTool;

impl ContrastTool {
    /// Create a new `ContrastTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContrastTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ContrastTool {
    fn name(&self) -> &str {
        "contrast"
    }

    fn description(&self) -> &str {
        "Computes the contrast ratio between two luminance values."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ContrastTool is a stub");
        Ok("Result from contrast".into())
    }
}
