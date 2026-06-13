//! Clipboard tool implementation.
//!
//! Provides a `ClipboardTool` that reads from and writes to the system
//! clipboard. Gated behind the `clipboard` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for reading from and writing to the system clipboard.
#[derive(Debug, Clone)]
pub struct ClipboardTool;

impl ClipboardTool {
    /// Create a new `ClipboardTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for ClipboardTool {
    fn name(&self) -> &str {
        "clipboard"
    }

    fn description(&self) -> &str {
        "Read from or write to the system clipboard. \
         Input should be 'read' to read, or 'write\\n<content>' to write."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("ClipboardTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
