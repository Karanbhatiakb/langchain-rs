//! Anagram checker tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks whether two strings are anagrams..
#[derive(Debug, Clone)]
pub struct AnagramTool;

impl AnagramTool {
    /// Create a new `AnagramTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnagramTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AnagramTool {
    fn name(&self) -> &str {
        "anagram"
    }

    fn description(&self) -> &str {
        "Checks whether two strings are anagrams."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("AnagramTool is a stub");
        Ok("Result from anagram".into())
    }
}
