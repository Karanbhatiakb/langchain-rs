//! Line count tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that counts the number of lines in a string.
#[derive(Debug, Clone)]
pub struct CountLinesTool;

impl CountLinesTool {
    /// Create a new `CountLinesTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CountLinesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CountLinesTool {
    fn name(&self) -> &str {
        "count_lines"
    }

    fn description(&self) -> &str {
        "Counts the number of lines in the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CountLinesTool is a stub");
        Ok("Result from count_lines".into())
    }
}
