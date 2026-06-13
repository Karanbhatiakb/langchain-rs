//! Prime check tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks whether a number is prime.
#[derive(Debug, Clone)]
pub struct PrimeCheckTool;

impl PrimeCheckTool {
    /// Create a new `PrimeCheckTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PrimeCheckTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for PrimeCheckTool {
    fn name(&self) -> &str {
        "prime_check"
    }

    fn description(&self) -> &str {
        "Checks whether the input number is a prime number."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("PrimeCheckTool is a stub");
        Ok("Result from prime_check".into())
    }
}
