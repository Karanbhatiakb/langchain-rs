//! NASA API toolkit for accessing space and earth data.

use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

/// Tool that fetches the Astronomy Picture of the Day from NASA.
#[derive(Debug)]
pub struct NASAApodTool;

#[async_trait]
impl BaseTool for NASAApodTool {
    fn name(&self) -> &str {
        "nasa_apod"
    }

    fn description(&self) -> &str {
        "Fetches the Astronomy Picture of the Day from NASA"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from nasa_apod (stub)".to_string())
    }
}

/// Tool that queries Mars Rover photos from NASA.
#[derive(Debug)]
pub struct NASAMarsRoverTool;

#[async_trait]
impl BaseTool for NASAMarsRoverTool {
    fn name(&self) -> &str {
        "nasa_mars_rover"
    }

    fn description(&self) -> &str {
        "Queries Mars Rover photos from NASA"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from nasa_mars_rover (stub)".to_string())
    }
}

/// A toolkit for accessing NASA APIs.
///
/// Provides tools for APOD, Mars Rover photos, and other NASA data.
#[derive(Debug)]
pub struct NASAToolkit;

impl NASAToolkit {
    /// Creates a new [`NASAToolkit`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for NASAToolkit {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseToolkit for NASAToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![
            Arc::new(NASAApodTool) as Arc<dyn BaseTool>,
            Arc::new(NASAMarsRoverTool),
        ]
    }

    fn name(&self) -> &str {
        "nasa"
    }
}
