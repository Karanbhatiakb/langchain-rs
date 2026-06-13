//! Sleep tool for pausing execution.

use async_trait::async_trait;
use super::traits::{BaseTool, ToolResult};

/// Tool that pauses execution for a specified duration.
#[derive(Debug, Clone)]
pub struct SleepTool;

impl SleepTool {
    /// Creates a new [`SleepTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for SleepTool {
    fn name(&self) -> &str {
        "sleep"
    }

    fn description(&self) -> &str {
        "Pauses execution for a specified duration"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok("Slept for 1 second".to_string())
    }
}
