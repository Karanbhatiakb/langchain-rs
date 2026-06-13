//! MultiOn AI browser control tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that controls browser actions using MultiOn AI.
#[derive(Debug)]
pub struct MultionTool;

impl MultionTool {
    /// Creates a new [`MultionTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for MultionTool {
    fn name(&self) -> &str {
        "multion"
    }

    fn description(&self) -> &str {
        "Controls browser actions using MultiOn AI"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "MultiOn API not configured (stub)".into(),
        ))
    }
}
