//! Statistical mode tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the statistical mode of a list of numbers.
#[derive(Debug, Clone)]
pub struct ModeTool;

impl ModeTool {
    /// Create a new `ModeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ModeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ModeTool {
    fn name(&self) -> &str {
        "mode"
    }

    fn description(&self) -> &str {
        "Computes the statistical mode (most frequent value) of a list of numbers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ModeTool is a stub");
        Ok("Result from mode".into())
    }
}
