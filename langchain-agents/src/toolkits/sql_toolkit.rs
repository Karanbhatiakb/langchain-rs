use async_trait::async_trait;
use langchain_tools::traits::{BaseTool, ToolResult};
use std::sync::Arc;

use crate::toolkits::traits::BaseToolkit;

#[derive(Debug)]
pub struct SQLQueryTool;

#[async_trait]
impl BaseTool for SQLQueryTool {
    fn name(&self) -> &str {
        "sql_query"
    }

    fn description(&self) -> &str {
        "Executes a SQL query"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from sql_query (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct SQLSchemaTool;

#[async_trait]
impl BaseTool for SQLSchemaTool {
    fn name(&self) -> &str {
        "sql_schema"
    }

    fn description(&self) -> &str {
        "Returns the schema of a SQL database"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from sql_schema (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct SQLCheckTool;

#[async_trait]
impl BaseTool for SQLCheckTool {
    fn name(&self) -> &str {
        "sql_check_query"
    }

    fn description(&self) -> &str {
        "Checks a SQL query for correctness"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Ok("Result from sql_check_query (stub)".to_string())
    }
}

#[derive(Debug)]
pub struct SQLToolkit;

impl SQLToolkit {
    pub fn new() -> Self {
        Self
    }
}

impl BaseToolkit for SQLToolkit {
    fn get_tools(&self) -> Vec<Arc<dyn BaseTool>> {
        vec![
            Arc::new(SQLQueryTool) as Arc<dyn BaseTool>,
            Arc::new(SQLSchemaTool) as Arc<dyn BaseTool>,
            Arc::new(SQLCheckTool),
        ]
    }

    fn name(&self) -> &str {
        "sql"
    }
}
