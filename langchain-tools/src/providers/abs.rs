//! Absolute value tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the absolute value of a number.
#[derive(Debug, Clone)]
pub struct AbsTool;

impl AbsTool {
    /// Create a new `AbsTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AbsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AbsTool {
    fn name(&self) -> &str {
        "abs"
    }

    fn description(&self) -> &str {
        "Returns the absolute value of the input number."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("AbsTool is a stub");
        Ok("Result from abs".into())
    }
}
