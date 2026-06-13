//! SQL database query tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that queries a SQL database.
#[derive(Debug)]
pub struct SQLDatabaseTool;

impl SQLDatabaseTool {
    /// Creates a new [`SQLDatabaseTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for SQLDatabaseTool {
    fn name(&self) -> &str {
        "sql_database"
    }

    fn description(&self) -> &str {
        "Queries a SQL database"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "SQL database not configured (stub)".into(),
        ))
    }
}
