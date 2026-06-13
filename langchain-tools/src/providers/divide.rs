//! Division math tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that divides two numbers.
#[derive(Debug, Clone)]
pub struct DivideTool;

impl DivideTool {
    /// Create a new `DivideTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DivideTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DivideTool {
    fn name(&self) -> &str {
        "divide"
    }

    fn description(&self) -> &str {
        "Divides two numbers. Input format: 'a b' returns a / b."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DivideTool is a stub");
        Ok("Result from divide".into())
    }
}
