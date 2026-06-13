//! String replace tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that replaces substrings in a string.
#[derive(Debug, Clone)]
pub struct ReplaceTool;

impl ReplaceTool {
    /// Create a new `ReplaceTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReplaceTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for ReplaceTool {
    fn name(&self) -> &str {
        "replace"
    }

    fn description(&self) -> &str {
        "Replaces occurrences of a substring with another string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ReplaceTool is a stub");
        Ok("Result from replace".into())
    }
}
