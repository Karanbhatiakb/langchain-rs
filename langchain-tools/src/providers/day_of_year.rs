//! Day of year tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the day of the year for a given date..
#[derive(Debug, Clone)]
pub struct DayOfYearTool;

impl DayOfYearTool {
    /// Create a new `DayOfYearTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DayOfYearTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DayOfYearTool {
    fn name(&self) -> &str {
        "day_of_year"
    }

    fn description(&self) -> &str {
        "Returns the day of the year for a given date."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DayOfYearTool is a stub");
        Ok("Result from day_of_year".into())
    }
}
