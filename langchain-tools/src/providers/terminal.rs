//! Terminal command tool implementation.
//!
//! Provides a `TerminalTool` that runs shell commands on the local system.
//! Gated behind the `terminal` feature flag. Use with caution.

use async_trait::async_trait;
use langchain_core::errors::ChainError;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for executing terminal / shell commands.
#[derive(Debug, Clone)]
pub struct TerminalTool;

impl TerminalTool {
    /// Create a new `TerminalTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for TerminalTool {
    fn name(&self) -> &str {
        "terminal"
    }

    fn description(&self) -> &str {
        "Execute a shell command on the local system and return its output. \
         Input should be a valid shell command string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("TerminalTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}

impl TerminalTool {
    fn sanitize_command(&self, _command: &str) -> Result<String, ChainError> {
        Ok(String::new())
    }
}
