//! Google Cloud APIs tool (Vision, Speech, etc.).

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that interacts with Google Cloud APIs (Vision, Speech, etc.).
#[derive(Debug)]
pub struct GoogleCloudTool;

impl GoogleCloudTool {
    /// Creates a new [`GoogleCloudTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GoogleCloudTool {
    fn name(&self) -> &str {
        "google_cloud"
    }

    fn description(&self) -> &str {
        "Interacts with Google Cloud APIs (Vision, Speech, etc.)"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Google Cloud not configured (stub)".into(),
        ))
    }
}
