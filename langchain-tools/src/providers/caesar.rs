//! Caesar cipher tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that applies the Caesar cipher to a string.
#[derive(Debug, Clone)]
pub struct CaesarTool;

impl CaesarTool {
    /// Create a new `CaesarTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CaesarTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CaesarTool {
    fn name(&self) -> &str {
        "caesar"
    }

    fn description(&self) -> &str {
        "Applies the Caesar cipher to the input string. Input format: 'shift:text'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CaesarTool is a stub");
        Ok("Result from caesar".into())
    }
}
