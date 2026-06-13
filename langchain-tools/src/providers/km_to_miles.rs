//! Kilometers to miles conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts kilometers to miles.
#[derive(Debug, Clone)]
pub struct KmToMilesTool;

impl KmToMilesTool {
    /// Create a new `KmToMilesTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for KmToMilesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for KmToMilesTool {
    fn name(&self) -> &str {
        "km_to_miles"
    }

    fn description(&self) -> &str {
        "Converts a distance from kilometers to miles."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("KmToMilesTool is a stub");
        Ok("Result from km_to_miles".into())
    }
}
