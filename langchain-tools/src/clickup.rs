//! ClickUp task management tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that manages tasks in ClickUp.
#[derive(Debug)]
pub struct ClickUpTool;

impl ClickUpTool {
    /// Creates a new [`ClickUpTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for ClickUpTool {
    fn name(&self) -> &str {
        "clickup"
    }

    fn description(&self) -> &str {
        "Manages tasks in ClickUp"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "ClickUp not configured (stub)".into(),
        ))
    }
}
