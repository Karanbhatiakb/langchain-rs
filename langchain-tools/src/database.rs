//! Database query tool.

use async_trait::async_trait;

use super::traits::{BaseTool, ToolResult};
use langchain_core::errors::ChainError;

pub struct SQLDatabaseTool {
    #[allow(dead_code)]
    connection_string: String,
}

impl SQLDatabaseTool {
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }
}

#[async_trait]
impl BaseTool for SQLDatabaseTool {
    fn name(&self) -> &str {
        "sql_database"
    }

    fn description(&self) -> &str {
        "Executes SQL queries against a database. Uses sqlx internally. The connection string should be set in the constructor. For direct query execution, use the QuerySQLTool. Input: SQL query to execute."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "SQLDatabaseTool requires the `database` feature and direct sqlx connection setup. Use ListTablesTool or QuerySQLTool for specific operations.".into(),
        ))
    }
}

pub struct ListTablesTool {
    #[allow(dead_code)]
    connection_string: String,
}

impl ListTablesTool {
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }
}

#[async_trait]
impl BaseTool for ListTablesTool {
    fn name(&self) -> &str {
        "list_tables"
    }

    fn description(&self) -> &str {
        "Lists all tables in the connected SQL database."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "ListTablesTool requires the `database` feature with sqlx enabled".into(),
        ))
    }
}

pub struct QuerySQLTool {
    #[allow(dead_code)]
    connection_string: String,
}

impl QuerySQLTool {
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }
}

#[async_trait]
impl BaseTool for QuerySQLTool {
    fn name(&self) -> &str {
        "query_sql"
    }

    fn description(&self) -> &str {
        "Executes a SQL query on the database. Input should be a valid SQL query."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        Err(ChainError::ToolError(
            "QuerySQLTool requires the `database` feature with sqlx enabled".into(),
        ))
    }
}
