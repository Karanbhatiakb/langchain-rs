//! Left strip tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that strips leading characters from a string.
#[derive(Debug, Clone)]
pub struct LstripTool;

impl LstripTool {
    /// Create a new `LstripTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LstripTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for LstripTool {
    fn name(&self) -> &str {
        "lstrip"
    }

    fn description(&self) -> &str {
        "Strips leading whitespace (or given characters) from the input."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("LstripTool is a stub");
        Ok("Result from lstrip".into())
    }
}
