//! Celsius to Fahrenheit conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts Celsius to Fahrenheit.
#[derive(Debug, Clone)]
pub struct CelsiusToFahrenheitTool;

impl CelsiusToFahrenheitTool {
    /// Create a new `CelsiusToFahrenheitTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CelsiusToFahrenheitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CelsiusToFahrenheitTool {
    fn name(&self) -> &str {
        "celsius_to_fahrenheit"
    }

    fn description(&self) -> &str {
        "Converts a temperature from Celsius to Fahrenheit."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CelsiusToFahrenheitTool is a stub");
        Ok("Result from celsius_to_fahrenheit".into())
    }
}
