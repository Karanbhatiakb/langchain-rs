//! Median tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the median of a list of numbers.
#[derive(Debug, Clone)]
pub struct MedianTool;

impl MedianTool {
    /// Create a new `MedianTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MedianTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for MedianTool {
    fn name(&self) -> &str {
        "median"
    }

    fn description(&self) -> &str {
        "Computes the median value from a list of numbers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("MedianTool is a stub");
        Ok("Result from median".into())
    }
}
