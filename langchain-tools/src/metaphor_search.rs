//! Metaphor AI web search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches the web using Metaphor AI.
#[derive(Debug)]
pub struct MetaphorSearchTool;

impl MetaphorSearchTool {
    /// Creates a new [`MetaphorSearchTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for MetaphorSearchTool {
    fn name(&self) -> &str {
        "metaphor_search"
    }

    fn description(&self) -> &str {
        "Searches the web using Metaphor AI"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Metaphor API not configured (stub)".into(),
        ))
    }
}
