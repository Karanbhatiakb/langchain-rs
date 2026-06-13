//! CSV query tool implementation.
//!
//! Provides a `CsvQueryTool` that queries a CSV file.
//! Gated behind the `csv_query` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for querying CSV files.
#[derive(Debug, Clone)]
pub struct CsvQueryTool;

impl CsvQueryTool {
    /// Create a new `CsvQueryTool`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for CsvQueryTool {
    fn name(&self) -> &str {
        "csv_query"
    }

    fn description(&self) -> &str {
        "Query a CSV file using a simple filter expression. \
         Input should be '<file_path>\\n<query>'."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("CsvQueryTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
