//! OpenWeatherMap tool for fetching current weather data.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that retrieves current weather data from the OpenWeatherMap API.
#[derive(Debug)]
pub struct OpenWeatherMapTool;

impl OpenWeatherMapTool {
    /// Creates a new [`OpenWeatherMapTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for OpenWeatherMapTool {
    fn name(&self) -> &str {
        "openweathermap"
    }

    fn description(&self) -> &str {
        "Gets current weather data from OpenWeatherMap API"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Weather API not configured (stub)".into(),
        ))
    }
}
