//! Nuclia AI search and indexing tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches and indexes data using Nuclia AI.
#[derive(Debug)]
pub struct NucliaTool;

impl NucliaTool {
    /// Creates a new [`NucliaTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for NucliaTool {
    fn name(&self) -> &str {
        "nuclia"
    }

    fn description(&self) -> &str {
        "Searches and indexes data using Nuclia AI"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Nuclia API not configured (stub)".into(),
        ))
    }
}
