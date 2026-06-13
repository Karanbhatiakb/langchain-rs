//! CSV to JSON conversion tool.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool that converts CSV data to JSON.
#[derive(Debug, Clone)]
pub struct CsvToJsonTool;

impl CsvToJsonTool {
    /// Create a new `CsvToJsonTool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CsvToJsonTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for CsvToJsonTool {
    fn name(&self) -> &str {
        "csv_to_json"
    }

    fn description(&self) -> &str {
        "Converts CSV-formatted data into a JSON array of objects."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CsvToJsonTool is a stub");
        Ok("Result from csv_to_json".into())
    }
}
