//! NASA API tool for space data.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that fetches data from NASA APIs (APOD, Mars rover, etc.).
#[derive(Debug)]
pub struct NASATool;

impl NASATool {
    /// Creates a new [`NASATool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for NASATool {
    fn name(&self) -> &str {
        "nasa"
    }

    fn description(&self) -> &str {
        "Fetches data from NASA APIs (APOD, Mars rover, etc.)"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "NASA API not configured (stub)".into(),
        ))
    }
}
