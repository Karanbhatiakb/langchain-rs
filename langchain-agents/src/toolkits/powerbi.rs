//! Power BI toolkit for querying and managing Power BI datasets.

use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

/// Tool that connects to a Power BI dataset.
#[derive(Debug)]
pub struct PowerBIConnectTool;

#[async_trait]
impl BaseTool for PowerBIConnectTool {
    fn name(&self) -> &str {
        "powerbi_connect"
    }

    fn description(&self) -> &str {
        "Connects to a Power BI dataset"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from powerbi_connect (stub)".to_string())
    }
}

/// Tool that queries a Power BI dataset using DAX or SQL.
#[derive(Debug)]
pub struct PowerBIQueryTool;

#[async_trait]
impl BaseTool for PowerBIQueryTool {
    fn name(&self) -> &str {
        "powerbi_query"
    }

    fn description(&self) -> &str {
        "Queries a Power BI dataset using DAX"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from powerbi_query (stub)".to_string())
    }
}

/// A toolkit for interacting with Microsoft Power BI.
///
/// Provides tools for connecting to datasets and running DAX queries.
#[derive(Debug)]
pub struct PowerBIToolkit;

impl PowerBIToolkit {
    /// Creates a new [`PowerBIToolkit`].
    pub fn new() -> Self {
        Self
    }
}

impl Default for PowerBIToolkit {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseToolkit for PowerBIToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![
            Arc::new(PowerBIConnectTool) as Arc<dyn BaseTool>,
            Arc::new(PowerBIQueryTool),
        ]
    }

    fn name(&self) -> &str {
        "powerbi"
    }
}
