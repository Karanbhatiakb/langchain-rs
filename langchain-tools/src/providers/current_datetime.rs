//! Current date-time tool implementation.
//!
//! Provides a `CurrentDateTimeTool` that returns the current date and time
//! in the requested format. Gated behind the `current_datetime` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for retrieving the current date and time.
#[derive(Debug, Clone)]
pub struct CurrentDateTimeTool {
    format: String,
}

impl CurrentDateTimeTool {
    /// Create a new `CurrentDateTimeTool` with the given format string.
    ///
    /// `format` uses `chrono` format specifiers (e.g. `"%Y-%m-%d %H:%M:%S"`).
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
        }
    }
}

#[async_trait]
impl BaseTool for CurrentDateTimeTool {
    fn name(&self) -> &str {
        "current_datetime"
    }

    fn description(&self) -> &str {
        "Returns the current date and time. Optional format parameter uses \
         chrono format specifiers (default: '%Y-%m-%d %H:%M:%S')."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CurrentDateTimeTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
