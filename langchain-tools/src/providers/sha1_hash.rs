//! SHA-1 hash tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the SHA-1 hash of a string.
#[derive(Debug, Clone)]
pub struct Sha1HashTool;

impl Sha1HashTool {
    /// Create a new `Sha1HashTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha1HashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for Sha1HashTool {
    fn name(&self) -> &str {
        "sha1_hash"
    }

    fn description(&self) -> &str {
        "Computes the SHA-1 hash of the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("Sha1HashTool is a stub");
        Ok("Result from sha1_hash".into())
    }
}
