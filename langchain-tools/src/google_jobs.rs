//! Google Jobs search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches for job listings using Google Jobs.
#[derive(Debug)]
pub struct GoogleJobsTool;

impl GoogleJobsTool {
    /// Creates a new [`GoogleJobsTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GoogleJobsTool {
    fn name(&self) -> &str {
        "google_jobs"
    }

    fn description(&self) -> &str {
        "Searches for job listings using Google Jobs"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Google Jobs API not configured (stub)".into(),
        ))
    }
}
