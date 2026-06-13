//! Fibonacci sequence tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the Fibonacci sequence up to a given count.
#[derive(Debug, Clone)]
pub struct FibonacciTool;

impl FibonacciTool {
    /// Create a new `FibonacciTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FibonacciTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FibonacciTool {
    fn name(&self) -> &str {
        "fibonacci"
    }

    fn description(&self) -> &str {
        "Computes the Fibonacci sequence up to the specified number of terms."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FibonacciTool is a stub");
        Ok("Result from fibonacci".into())
    }
}
