//! Neo4j graph database connector.
//!
//! Provides a [`Neo4jGraph`] struct for connecting to and querying a Neo4j
//! database using the Bolt protocol.

use langchain_core::errors::{ChainError, Result};

/// A connector for interacting with a Neo4j graph database.
///
/// Manages a connection to a Neo4j instance and provides methods for
/// executing Cypher queries, adding triples, and introspecting the schema.
///
/// # Stub
///
/// This is a stub implementation. Wire up the `neo4j` driver crate to
/// enable live graph operations.
#[derive(Debug, Clone)]
pub struct Neo4jGraph {
    /// The Bolt URI of the Neo4j instance.
    pub uri: String,
    /// The database user.
    pub user: String,
    /// The database password.
    pub password: String,
    /// The target database name.
    pub database: String,
}

impl Neo4jGraph {
    /// Creates a new [`Neo4jGraph`] connector.
    pub fn new(uri: &str, user: &str, password: &str) -> Self {
        Self {
            uri: uri.to_string(),
            user: user.to_string(),
            password: password.to_string(),
            database: "neo4j".to_string(),
        }
    }

    /// Sets the target database and returns self (builder pattern).
    pub fn with_database(mut self, database: &str) -> Self {
        self.database = database.to_string();
        self
    }

    /// Executes a Cypher query against the Neo4j database.
    pub async fn query(&self, _cypher: &str) -> Result<serde_json::Value> {
        Err(ChainError::ToolError(
            "Neo4j not configured (stub) — add the neo4j driver crate and connection logic".into(),
        ))
    }

    /// Adds a triple (subject, predicate, object) to the graph.
    pub async fn add_triple(
        &self,
        _subject: &str,
        _predicate: &str,
        _object: &str,
    ) -> Result<()> {
        Err(ChainError::ToolError(
            "Neo4j not configured (stub)".into(),
        ))
    }

    /// Refreshes the schema by introspecting the graph.
    pub async fn refresh_schema(&self) -> Result<()> {
        Err(ChainError::ToolError(
            "Neo4j not configured (stub)".into(),
        ))
    }
}
