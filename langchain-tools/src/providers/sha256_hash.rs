//! SHA-256 hash tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the SHA-256 hash of a string.
#[derive(Debug, Clone)]
pub struct Sha256HashTool;

impl Sha256HashTool {
    /// Create a new `Sha256HashTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha256HashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for Sha256HashTool {
    fn name(&self) -> &str {
        "sha256_hash"
    }

    fn description(&self) -> &str {
        "Computes the SHA-256 hash of the input string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("Sha256HashTool is a stub");
        Ok("Result from sha256_hash".into())
    }
}
