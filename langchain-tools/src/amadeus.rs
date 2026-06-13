//! Amadeus flight search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches for flight information using the Amadeus API.
#[derive(Debug)]
pub struct AmadeusTool;

impl AmadeusTool {
    /// Creates a new [`AmadeusTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for AmadeusTool {
    fn name(&self) -> &str {
        "amadeus"
    }

    fn description(&self) -> &str {
        "Searches for flight information using Amadeus API"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Amadeus API not configured (stub)".into(),
        ))
    }
}
