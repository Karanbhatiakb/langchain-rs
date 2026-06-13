//! Fahrenheit to Celsius conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts Fahrenheit to Celsius.
#[derive(Debug, Clone)]
pub struct FahrenheitToCelsiusTool;

impl FahrenheitToCelsiusTool {
    /// Create a new `FahrenheitToCelsiusTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FahrenheitToCelsiusTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for FahrenheitToCelsiusTool {
    fn name(&self) -> &str {
        "fahrenheit_to_celsius"
    }

    fn description(&self) -> &str {
        "Converts a temperature from Fahrenheit to Celsius."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("FahrenheitToCelsiusTool is a stub");
        Ok("Result from fahrenheit_to_celsius".into())
    }
}
