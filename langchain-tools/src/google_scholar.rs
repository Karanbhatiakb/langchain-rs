//! Google Scholar academic paper search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches Google Scholar for academic papers.
#[derive(Debug)]
pub struct GoogleScholarTool;

impl GoogleScholarTool {
    /// Creates a new [`GoogleScholarTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GoogleScholarTool {
    fn name(&self) -> &str {
        "google_scholar"
    }

    fn description(&self) -> &str {
        "Searches Google Scholar for academic papers"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Google Scholar not configured (stub)".into(),
        ))
    }
}
