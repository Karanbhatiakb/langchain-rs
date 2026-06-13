//! Cosine trigonometric tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes the cosine of an angle.
#[derive(Debug, Clone)]
pub struct CosTool;

impl CosTool {
    /// Create a new `CosTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CosTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CosTool {
    fn name(&self) -> &str {
        "cos"
    }

    fn description(&self) -> &str {
        "Computes the cosine of an angle (in radians). Input: a single number."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CosTool is a stub");
        Ok("Result from cos".into())
    }
}
