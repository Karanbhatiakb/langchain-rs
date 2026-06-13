//! Stack Exchange search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches Stack Exchange sites for answers.
#[derive(Debug)]
pub struct StackExchangeTool;

impl StackExchangeTool {
    /// Creates a new [`StackExchangeTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for StackExchangeTool {
    fn name(&self) -> &str {
        "stackexchange"
    }

    fn description(&self) -> &str {
        "Searches Stack Exchange sites for answers"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Stack Exchange not configured (stub)".into(),
        ))
    }
}
