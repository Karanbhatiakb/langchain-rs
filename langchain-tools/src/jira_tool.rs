//! Jira issue tracking tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that interacts with Jira for issue tracking.
#[derive(Debug)]
pub struct JiraTool;

impl JiraTool {
    /// Creates a new [`JiraTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for JiraTool {
    fn name(&self) -> &str {
        "jira"
    }

    fn description(&self) -> &str {
        "Interacts with Jira for issue tracking"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Jira API not configured (stub)".into(),
        ))
    }
}
