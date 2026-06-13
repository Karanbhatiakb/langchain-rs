//! DNS lookup tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that performs a DNS lookup for a given hostname.
#[derive(Debug, Clone)]
pub struct DnsLookupTool;

impl DnsLookupTool {
    /// Create a new `DnsLookupTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DnsLookupTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for DnsLookupTool {
    fn name(&self) -> &str {
        "dns_lookup"
    }

    fn description(&self) -> &str {
        "Resolves the IP address(es) for a given hostname via DNS."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DnsLookupTool is a stub");
        Ok("Result from dns_lookup".into())
    }
}
