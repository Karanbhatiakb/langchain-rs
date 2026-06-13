//! Google Serper API web search tool.
//!
//! Provides a tool that executes searches through the Serper.dev Google
//! Search API and returns structured results.

use async_trait::async_trait;
use super::traits::{BaseTool, ToolResult};

/// Tool that performs web searches via the Serper.dev Google Search API.
///
/// Requires the `SERPER_API_KEY` environment variable to be set.
///
/// # Stub
///
/// This is a stub implementation. Provide a valid API key and configure
/// the HTTP client to enable live searches.
#[derive(Debug)]
pub struct GoogleSerperTool;

impl GoogleSerperTool {
    /// Creates a new [`GoogleSerperTool`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoogleSerperTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for GoogleSerperTool {
    fn name(&self) -> &str {
        "google_serper"
    }

    fn description(&self) -> &str {
        "Performs web searches using the Serper.dev Google Search API"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from google_serper (stub)".to_string())
    }
}
