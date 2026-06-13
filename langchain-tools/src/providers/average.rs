//! Average tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the average (mean) of a list of numbers.
#[derive(Debug, Clone)]
pub struct AverageTool;

impl AverageTool {
    /// Create a new `AverageTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AverageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AverageTool {
    fn name(&self) -> &str {
        "average"
    }

    fn description(&self) -> &str {
        "Computes the arithmetic mean of a list of numbers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("AverageTool is a stub");
        Ok("Result from average".into())
    }
}
