//! Word count tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that counts the number of words in a string.
#[derive(Debug, Clone)]
pub struct CountWordsTool;

impl CountWordsTool {
    /// Create a new `CountWordsTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CountWordsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CountWordsTool {
    fn name(&self) -> &str {
        "count_words"
    }

    fn description(&self) -> &str {
        "Counts the number of words in the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CountWordsTool is a stub");
        Ok("Result from count_words".into())
    }
}
