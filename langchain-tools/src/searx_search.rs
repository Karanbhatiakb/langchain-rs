//! Searx search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches using a Searx instance.
#[derive(Debug)]
pub struct SearxSearchTool;

impl SearxSearchTool {
    /// Creates a new [`SearxSearchTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for SearxSearchTool {
    fn name(&self) -> &str {
        "searx_search"
    }

    fn description(&self) -> &str {
        "Searches using a Searx instance"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Searx search not available (stub)".into(),
        ))
    }
}
