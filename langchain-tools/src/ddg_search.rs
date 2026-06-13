//! DuckDuckGo search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches DuckDuckGo for query results.
#[derive(Debug)]
pub struct DuckDuckGoSearchTool;

impl DuckDuckGoSearchTool {
    /// Creates a new [`DuckDuckGoSearchTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for DuckDuckGoSearchTool {
    fn name(&self) -> &str {
        "duckduckgo_search"
    }

    fn description(&self) -> &str {
        "Searches DuckDuckGo for query results"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Search results not available (stub)".into(),
        ))
    }
}
