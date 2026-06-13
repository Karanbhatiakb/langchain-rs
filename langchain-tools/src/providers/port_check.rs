//! Port check tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that checks whether a TCP port is open on a host.
#[derive(Debug, Clone)]
pub struct PortCheckTool;

impl PortCheckTool {
    /// Create a new `PortCheckTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PortCheckTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for PortCheckTool {
    fn name(&self) -> &str {
        "port_check"
    }

    fn description(&self) -> &str {
        "Checks whether a TCP port is open on the specified host."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("PortCheckTool is a stub");
        Ok("Result from port_check".into())
    }
}
