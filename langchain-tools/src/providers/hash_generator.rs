//! Hash generator tool implementation.
//!
//! Provides a `HashGeneratorTool` that generates hashes (MD5, SHA-256) of
//! input strings. Gated behind the `hash_generator` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for generating hashes of input strings.
#[derive(Debug, Clone)]
pub struct HashGeneratorTool;

impl HashGeneratorTool {
    /// Create a new `HashGeneratorTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HashGeneratorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for HashGeneratorTool {
    fn name(&self) -> &str {
        "hash_generator"
    }

    fn description(&self) -> &str {
        "Generates hashes (MD5, SHA-256) of the input string. Input format: \
         'md5:<text>' or 'sha256:<text>'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("HashGeneratorTool is a stub");
        Ok("hash_stub".into())
    }
}
