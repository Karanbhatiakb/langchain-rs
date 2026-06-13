//! Acoustic fingerprint tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that analyzes acoustic fingerprints of audio input..
#[derive(Debug, Clone)]
pub struct AcousticTool;

impl AcousticTool {
    /// Create a new `AcousticTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AcousticTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for AcousticTool {
    fn name(&self) -> &str {
        "acoustic"
    }

    fn description(&self) -> &str {
        "Analyzes acoustic fingerprints of audio input."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("AcousticTool is a stub");
        Ok("Result from acoustic".into())
    }
}
