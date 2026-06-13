//! YouTube video search and information tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches and fetches YouTube video information.
#[derive(Debug)]
pub struct YouTubeTool;

impl YouTubeTool {
    /// Creates a new [`YouTubeTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for YouTubeTool {
    fn name(&self) -> &str {
        "youtube"
    }

    fn description(&self) -> &str {
        "Searches and fetches YouTube video information"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "YouTube API not configured (stub)".into(),
        ))
    }
}
