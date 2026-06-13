//! URL parse tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that parses a URL into its components.
#[derive(Debug, Clone)]
pub struct UrlParseTool;

impl UrlParseTool {
    /// Create a new `UrlParseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for UrlParseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for UrlParseTool {
    fn name(&self) -> &str {
        "url_parse"
    }

    fn description(&self) -> &str {
        "Parses a URL and returns its components (scheme, host, path, etc.)."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("UrlParseTool is a stub");
        Ok("Result from url_parse".into())
    }
}
