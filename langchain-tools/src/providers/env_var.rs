//! Environment variable tool implementation.
//!
//! Provides an `EnvVarTool` that reads environment variables.
//! Gated behind the `env_var` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for reading environment variables.
#[derive(Debug, Clone)]
pub struct EnvVarTool;

impl EnvVarTool {
    /// Create a new `EnvVarTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for EnvVarTool {
    fn name(&self) -> &str {
        "env_var"
    }

    fn description(&self) -> &str {
        "Read the value of an environment variable. \
         Input should be the name of the environment variable."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("EnvVarTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
