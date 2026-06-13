//! IP address tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that retrieves the public IP address of the current machine.
#[derive(Debug, Clone)]
pub struct IpAddressTool;

impl IpAddressTool {
    /// Create a new `IpAddressTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for IpAddressTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for IpAddressTool {
    fn name(&self) -> &str {
        "ip_address"
    }

    fn description(&self) -> &str {
        "Returns the public IP address of the current machine."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("IpAddressTool is a stub");
        Ok("Result from ip_address".into())
    }
}
