//! Twitter/X search and posting tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches and posts on Twitter/X.
#[derive(Debug)]
pub struct TwitterTool;

impl TwitterTool {
    /// Creates a new [`TwitterTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for TwitterTool {
    fn name(&self) -> &str {
        "twitter"
    }

    fn description(&self) -> &str {
        "Searches and posts on Twitter/X"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Twitter API not configured (stub)".into(),
        ))
    }
}
