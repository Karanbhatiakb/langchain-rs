//! Slugify tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a string to a URL-friendly slug.
#[derive(Debug, Clone)]
pub struct SlugifyTool;

impl SlugifyTool {
    /// Create a new `SlugifyTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SlugifyTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for SlugifyTool {
    fn name(&self) -> &str {
        "slugify"
    }

    fn description(&self) -> &str {
        "Converts a string into a URL-friendly slug."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SlugifyTool is a stub");
        Ok("Result from slugify".into())
    }
}
