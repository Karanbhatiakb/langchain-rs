//! Day of week tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the day of the week for a given date..
#[derive(Debug, Clone)]
pub struct DayOfWeekTool;

impl DayOfWeekTool {
    /// Create a new `DayOfWeekTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DayOfWeekTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DayOfWeekTool {
    fn name(&self) -> &str {
        "day_of_week"
    }

    fn description(&self) -> &str {
        "Returns the day of the week for a given date."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DayOfWeekTool is a stub");
        Ok("Result from day_of_week".into())
    }
}
