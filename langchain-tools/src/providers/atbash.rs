//! Atbash cipher tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that applies the Atbash cipher to a string.
#[derive(Debug, Clone)]
pub struct AtbashTool;

impl AtbashTool {
    /// Create a new `AtbashTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AtbashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AtbashTool {
    fn name(&self) -> &str {
        "atbash"
    }

    fn description(&self) -> &str {
        "Applies the Atbash cipher (reverse alphabet substitution) to the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("AtbashTool is a stub");
        Ok("Result from atbash".into())
    }
}
