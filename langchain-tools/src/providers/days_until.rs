//! Days until date tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that calculates the number of days until a target date..
#[derive(Debug, Clone)]
pub struct DaysUntilTool;

impl DaysUntilTool {
    /// Create a new `DaysUntilTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DaysUntilTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DaysUntilTool {
    fn name(&self) -> &str {
        "days_until"
    }

    fn description(&self) -> &str {
        "Calculates the number of days until a target date."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DaysUntilTool is a stub");
        Ok("Result from days_until".into())
    }
}
