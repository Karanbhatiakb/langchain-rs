//! Title case string tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts a string to title case.
#[derive(Debug, Clone)]
pub struct TitleCaseTool;

impl TitleCaseTool {
    /// Create a new `TitleCaseTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TitleCaseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for TitleCaseTool {
    fn name(&self) -> &str {
        "title_case"
    }

    fn description(&self) -> &str {
        "Converts the input string to title case (capitalizes the first letter of each word)."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("TitleCaseTool is a stub");
        Ok("Result from title_case".into())
    }
}
