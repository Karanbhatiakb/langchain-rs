//! Right strip tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that strips trailing characters from a string.
#[derive(Debug, Clone)]
pub struct RstripTool;

impl RstripTool {
    /// Create a new `RstripTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RstripTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for RstripTool {
    fn name(&self) -> &str {
        "rstrip"
    }

    fn description(&self) -> &str {
        "Strips trailing whitespace (or given characters) from the input."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("RstripTool is a stub");
        Ok("Result from rstrip".into())
    }
}
