//! String concatenation tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that concatenates multiple strings.
#[derive(Debug, Clone)]
pub struct ConcatTool;

impl ConcatTool {
    /// Create a new `ConcatTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConcatTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ConcatTool {
    fn name(&self) -> &str {
        "concat"
    }

    fn description(&self) -> &str {
        "Concatenates multiple strings together."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ConcatTool is a stub");
        Ok("Result from concat".into())
    }
}
