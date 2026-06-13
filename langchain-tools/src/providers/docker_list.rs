//! Docker list tool implementation.
//!
//! Provides a `DockerListTool` that lists Docker containers.
//! Gated behind the `docker_list` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for listing Docker containers.
#[derive(Debug, Clone)]
pub struct DockerListTool;

impl DockerListTool {
    /// Create a new `DockerListTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for DockerListTool {
    fn name(&self) -> &str {
        "docker_list"
    }

    fn description(&self) -> &str {
        "List Docker containers on the host system. Input can optionally \
         filter by status (e.g. 'running', 'all')."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DockerListTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
