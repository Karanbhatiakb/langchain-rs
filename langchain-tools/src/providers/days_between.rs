//! Days between dates tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that calculates the number of days between two dates..
#[derive(Debug, Clone)]
pub struct DaysBetweenTool;

impl DaysBetweenTool {
    /// Create a new `DaysBetweenTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DaysBetweenTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DaysBetweenTool {
    fn name(&self) -> &str {
        "days_between"
    }

    fn description(&self) -> &str {
        "Calculates the number of days between two dates."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DaysBetweenTool is a stub");
        Ok("Result from days_between".into())
    }
}
