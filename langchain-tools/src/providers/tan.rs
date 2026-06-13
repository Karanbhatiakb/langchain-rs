//! Tangent trigonometric tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the tangent of an angle.
#[derive(Debug, Clone)]
pub struct TanTool;

impl TanTool {
    /// Create a new `TanTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for TanTool {
    fn name(&self) -> &str {
        "tan"
    }

    fn description(&self) -> &str {
        "Computes the tangent of an angle (in radians). Input: a single number."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("TanTool is a stub");
        Ok("Result from tan".into())
    }
}
