//! Shell command execution tool.

use std::time::Duration;

use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::timeout;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct ShellTool {
    timeout_secs: u64,
}

impl Default for ShellTool {
    fn default() -> Self {
        Self { timeout_secs: 30 }
    }
}

impl ShellTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

#[async_trait]
impl BaseTool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Executes shell commands. Input should be a valid shell command."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let result = timeout(
            Duration::from_secs(self.timeout_secs),
            Command::new("sh").arg("-c").arg(input).output(),
        )
        .await
        .map_err(|_| ChainError::ToolError("Shell command timed out".into()))?
        .map_err(|e| ChainError::ToolError(format!("Failed to run shell command: {}", e)))?;

        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            Err(ChainError::ToolError(format!("Shell error: {}", stderr)))
        }
    }
}
