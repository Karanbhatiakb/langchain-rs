//! Google Places search tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that searches for places using the Google Places API.
#[derive(Debug)]
pub struct GooglePlacesTool;

impl GooglePlacesTool {
    /// Creates a new [`GooglePlacesTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GooglePlacesTool {
    fn name(&self) -> &str {
        "google_places"
    }

    fn description(&self) -> &str {
        "Searches for places using Google Places API"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Google Places API not configured (stub)".into(),
        ))
    }
}
