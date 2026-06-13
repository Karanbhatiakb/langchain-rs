//! Days in month tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns the number of days in a given month and year..
#[derive(Debug, Clone)]
pub struct DaysInMonthTool;

impl DaysInMonthTool {
    /// Create a new `DaysInMonthTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DaysInMonthTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DaysInMonthTool {
    fn name(&self) -> &str {
        "days_in_month"
    }

    fn description(&self) -> &str {
        "Returns the number of days in a given month and year."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DaysInMonthTool is a stub");
        Ok("Result from days_in_month".into())
    }
}
