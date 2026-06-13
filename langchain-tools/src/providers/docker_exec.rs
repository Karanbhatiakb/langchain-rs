//! Docker exec tool implementation.
//!
//! Provides a `DockerExecTool` that executes a command inside a Docker
//! container. Gated behind the `docker_exec` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for executing commands inside a Docker container.
#[derive(Debug, Clone)]
pub struct DockerExecTool;

impl DockerExecTool {
    /// Create a new `DockerExecTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for DockerExecTool {
    fn name(&self) -> &str {
        "docker_exec"
    }

    fn description(&self) -> &str {
        "Execute a command inside a Docker container. \
         Input should be '<container_id> <command>'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("DockerExecTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
