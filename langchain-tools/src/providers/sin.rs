//! Sine trigonometric tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the sine of an angle.
#[derive(Debug, Clone)]
pub struct SinTool;

impl SinTool {
    /// Create a new `SinTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SinTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for SinTool {
    fn name(&self) -> &str {
        "sin"
    }

    fn description(&self) -> &str {
        "Computes the sine of an angle (in radians). Input: a single number."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SinTool is a stub");
        Ok("Result from sin".into())
    }
}
