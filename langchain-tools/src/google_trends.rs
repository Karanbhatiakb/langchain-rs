//! Google Trends tool for trending topics.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that fetches trending topics from Google Trends.
#[derive(Debug)]
pub struct GoogleTrendsTool;

impl GoogleTrendsTool {
    /// Creates a new [`GoogleTrendsTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GoogleTrendsTool {
    fn name(&self) -> &str {
        "google_trends"
    }

    fn description(&self) -> &str {
        "Fetches trending topics from Google Trends"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Google Trends not configured (stub)".into(),
        ))
    }
}
