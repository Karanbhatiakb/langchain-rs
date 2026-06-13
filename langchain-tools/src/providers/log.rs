//! Logarithm math tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the logarithm of a number.
#[derive(Debug, Clone)]
pub struct LogTool;

impl LogTool {
    /// Create a new `LogTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LogTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for LogTool {
    fn name(&self) -> &str {
        "log"
    }

    fn description(&self) -> &str {
        "Computes the logarithm of a number. Input format: 'value' for natural log, or 'value base' for arbitrary base."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("LogTool is a stub");
        Ok("Result from log".into())
    }
}
