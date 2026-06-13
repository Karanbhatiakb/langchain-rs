//! Greatest common divisor tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the greatest common divisor of two numbers.
#[derive(Debug, Clone)]
pub struct GcdTool;

impl GcdTool {
    /// Create a new `GcdTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GcdTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for GcdTool {
    fn name(&self) -> &str {
        "gcd"
    }

    fn description(&self) -> &str {
        "Computes the greatest common divisor of two integers."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("GcdTool is a stub");
        Ok("Result from gcd".into())
    }
}
