//! Merriam-Webster dictionary tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that looks up word definitions from Merriam-Webster.
#[derive(Debug)]
pub struct MerriamWebsterTool;

impl MerriamWebsterTool {
    /// Creates a new [`MerriamWebsterTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for MerriamWebsterTool {
    fn name(&self) -> &str {
        "merriam_webster"
    }

    fn description(&self) -> &str {
        "Looks up word definitions from Merriam-Webster"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Merriam-Webster API not configured (stub)".into(),
        ))
    }
}
