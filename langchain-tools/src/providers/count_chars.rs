//! Character count tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that counts the number of characters in a string.
#[derive(Debug, Clone)]
pub struct CountCharsTool;

impl CountCharsTool {
    /// Create a new `CountCharsTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CountCharsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CountCharsTool {
    fn name(&self) -> &str {
        "count_chars"
    }

    fn description(&self) -> &str {
        "Counts the number of characters (including whitespace) in the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CountCharsTool is a stub");
        Ok("Result from count_chars".into())
    }
}
