//! Google Lens visual search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that performs visual search using Google Lens.
#[derive(Debug)]
pub struct GoogleLensTool;

impl GoogleLensTool {
    /// Creates a new [`GoogleLensTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GoogleLensTool {
    fn name(&self) -> &str {
        "google_lens"
    }

    fn description(&self) -> &str {
        "Performs visual search using Google Lens"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Google Lens API not configured (stub)".into(),
        ))
    }
}
