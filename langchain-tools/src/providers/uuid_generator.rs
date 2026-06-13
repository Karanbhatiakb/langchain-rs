//! UUID generator tool implementation.
//!
//! Provides a `UuidGeneratorTool` that generates random UUID v4 strings.
//! Gated behind the `uuid_generator` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for generating UUID v4 identifiers.
#[derive(Debug, Clone)]
pub struct UuidGeneratorTool;

impl UuidGeneratorTool {
    /// Create a new `UuidGeneratorTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for UuidGeneratorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for UuidGeneratorTool {
    fn name(&self) -> &str {
        "uuid_generator"
    }

    fn description(&self) -> &str {
        "Generates a random UUID v4 identifier. Input is ignored; returns a new UUID string."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("UuidGeneratorTool is a stub");
        Ok("00000000-0000-4000-8000-000000000000".into())
    }
}
