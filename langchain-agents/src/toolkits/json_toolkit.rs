use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct JSONQueryTool;

#[async_trait]
impl BaseTool for JSONQueryTool {
    fn name(&self) -> &str {
        "json_query"
    }

    fn description(&self) -> &str {
        "Queries a JSON document"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from json_query (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct JSONSchemaTool;

#[async_trait]
impl BaseTool for JSONSchemaTool {
    fn name(&self) -> &str {
        "json_schema"
    }

    fn description(&self) -> &str {
        "Returns the schema of a JSON document"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from json_schema (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct JSONToolkit;

impl JSONToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl BaseToolkit for JSONToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![Arc::new(JSONQueryTool) as Arc<dyn BaseTool>, Arc::new(JSONSchemaTool)]
    }

    fn name(&self) -> &str {
        "json"
    }
}
