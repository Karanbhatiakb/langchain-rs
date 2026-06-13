//! SQL connect tool implementation.
//!
//! Provides a `SQLConnectTool` that executes SQL queries against a configured
//! database. Gated behind the `sql_connect` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for executing SQL queries against a database.
#[derive(Debug, Clone)]
pub struct SQLConnectTool {
    connection_string: String,
}

impl SQLConnectTool {
    /// Create a new `SQLConnectTool` with the given connection string.
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }
}

#[async_trait]
impl BaseTool for SQLConnectTool {
    fn name(&self) -> &str {
        "sql_connect"
    }

    fn description(&self) -> &str {
        "Execute a SQL query against the configured database and return the \
         result as text. Input should be a valid SQL statement."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("SQLConnectTool is a stub; invoke returns empty");
        Ok(String::new())
    }
}
