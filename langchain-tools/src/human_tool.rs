//! Human-in-the-loop input tool.

use async_trait::async_trait;
use super::traits::{BaseTool, ToolResult};

/// Tool that asks a human for input, useful for human-in-the-loop workflows.
#[derive(Debug)]
pub struct HumanInputTool;

impl HumanInputTool {
    /// Creates a new [`HumanInputTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for HumanInputTool {
    fn name(&self) -> &str {
        "human_input"
    }

    fn description(&self) -> &str {
        "Asks a human for input (useful for human-in-the-loop)"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Human input not available in automated mode (stub)".to_string())
    }
}
