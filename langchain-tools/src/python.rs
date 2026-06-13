//! Python code execution tool.

use std::time::Duration;

use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::timeout;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct PythonREPLTool {
    python_path: String,
    timeout_secs: u64,
}

impl Default for PythonREPLTool {
    fn default() -> Self {
        Self {
            python_path: "python3".to_string(),
            timeout_secs: 30,
        }
    }
}

impl PythonREPLTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_python_path(mut self, path: impl Into<String>) -> Self {
        self.python_path = path.into();
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

#[async_trait]
impl BaseTool for PythonREPLTool {
    fn name(&self) -> &str {
        "python_repl"
    }

    fn description(&self) -> &str {
        "A Python shell. Use this to execute Python code. Input should be valid Python code."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let result = timeout(
            Duration::from_secs(self.timeout_secs),
            Command::new(&self.python_path)
                .arg("-c")
                .arg(input)
                .output(),
        )
        .await
        .map_err(|_| ChainError::ToolError("Python execution timed out".into()))?
        .map_err(|e| ChainError::ToolError(format!("Failed to run Python: {}", e)))?;

        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            Err(ChainError::ToolError(format!("Python error: {}", stderr)))
        }
    }
}
