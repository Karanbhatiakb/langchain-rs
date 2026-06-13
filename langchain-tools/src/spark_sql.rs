//! Spark SQL database tool.

use async_trait::async_trait;
use langchain_core::errors::ChainError;
use super::traits::{BaseTool, ToolResult};

/// Tool that queries a Spark SQL database.
#[derive(Debug)]
pub struct SparkSQLTool;

impl SparkSQLTool {
    /// Creates a new [`SparkSQLTool`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for SparkSQLTool {
    fn name(&self) -> &str {
        "spark_sql"
    }

    fn description(&self) -> &str {
        "Queries a Spark SQL database"
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "Spark SQL not configured (stub)".into(),
        ))
    }
}
