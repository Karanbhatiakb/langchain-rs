use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct CSVLookupTool;

#[async_trait]
impl BaseTool for CSVLookupTool {
    fn name(&self) -> &str {
        "csv_lookup"
    }

    fn description(&self) -> &str {
        "Looks up data in a CSV file"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from csv_lookup (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct CSVListTool;

#[async_trait]
impl BaseTool for CSVListTool {
    fn name(&self) -> &str {
        "csv_list"
    }

    fn description(&self) -> &str {
        "Lists columns in a CSV file"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from csv_list (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct CSVToolkit;

impl CSVToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl BaseToolkit for CSVToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![Arc::new(CSVLookupTool) as Arc<dyn BaseTool>, Arc::new(CSVListTool)]
    }

    fn name(&self) -> &str {
        "csv"
    }
}
