//! Calendar events tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that returns calendar events for a given date range..
#[derive(Debug, Clone)]
pub struct CalendarTool;

impl CalendarTool {
    /// Create a new `CalendarTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CalendarTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CalendarTool {
    fn name(&self) -> &str {
        "calendar"
    }

    fn description(&self) -> &str {
        "Returns calendar events for a given date range."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CalendarTool is a stub");
        Ok("Result from calendar".into())
    }
}
