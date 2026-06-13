//! Incrementing counter tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns an incrementing counter value.
#[derive(Debug, Clone)]
pub struct CounterTool;

impl CounterTool {
    /// Create a new `CounterTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CounterTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CounterTool {
    fn name(&self) -> &str {
        "counter"
    }

    fn description(&self) -> &str {
        "Returns an incrementing counter value. Each call increments by 1, starting from 0."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CounterTool is a stub");
        Ok("Result from counter".into())
    }
}
