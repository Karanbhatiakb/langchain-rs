//! Domain extract tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that extracts the domain name from a URL..
#[derive(Debug, Clone)]
pub struct DomainExtractTool;

impl DomainExtractTool {
    /// Create a new `DomainExtractTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DomainExtractTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DomainExtractTool {
    fn name(&self) -> &str {
        "domain_extract"
    }

    fn description(&self) -> &str {
        "Extracts the domain name from a URL."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DomainExtractTool is a stub");
        Ok("Result from domain_extract".into())
    }
}
