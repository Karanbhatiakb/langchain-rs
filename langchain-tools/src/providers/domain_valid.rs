//! Domain validate tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that validates whether a given string is a valid domain name..
#[derive(Debug, Clone)]
pub struct DomainValidTool;

impl DomainValidTool {
    /// Create a new `DomainValidTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DomainValidTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DomainValidTool {
    fn name(&self) -> &str {
        "domain_valid"
    }

    fn description(&self) -> &str {
        "Validates whether a given string is a valid domain name."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DomainValidTool is a stub");
        Ok("Result from domain_valid".into())
    }
}
