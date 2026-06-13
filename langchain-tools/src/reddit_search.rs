//! Reddit search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches Reddit for posts and comments.
#[derive(Debug)]
pub struct RedditSearchTool;

impl RedditSearchTool {
    /// Creates a new [`RedditSearchTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for RedditSearchTool {
    fn name(&self) -> &str {
        "reddit_search"
    }

    fn description(&self) -> &str {
        "Searches Reddit for posts and comments"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Reddit search not available (stub)".into(),
        ))
    }
}
