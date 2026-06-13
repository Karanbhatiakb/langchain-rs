//! Age calculator tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that calculates age from a birth date string..
#[derive(Debug, Clone)]
pub struct AgeTool;

impl AgeTool {
    /// Create a new `AgeTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AgeTool {
    fn name(&self) -> &str {
        "age"
    }

    fn description(&self) -> &str {
        "Calculates age from a birth date string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("AgeTool is a stub");
        Ok("Result from age".into())
    }
}
