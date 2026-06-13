//! BMI calculator tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that computes Body Mass Index from height and weight values..
#[derive(Debug, Clone)]
pub struct BmiTool;

impl BmiTool {
    /// Create a new `BmiTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BmiTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BmiTool {
    fn name(&self) -> &str {
        "bmi"
    }

    fn description(&self) -> &str {
        "Computes Body Mass Index from height and weight values."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("BmiTool is a stub");
        Ok("Result from bmi".into())
    }
}
