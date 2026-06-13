//! NetworkX (rustworkx) graph connector.
//!
//! Provides a [`NetworkXGraph`] struct that wraps an in-memory graph
//! using the `rustworkx` (Rust port of NetworkX) library.

use langchain_core::errors::{ChainError, Result};
use std::collections::HashMap;

/// A connector that wraps an in-memory graph, similar to Python's NetworkX.
///
/// In Rust the equivalent is `rustworkx`. This stub provides the same
/// interface for creating and analysing graphs.
///
/// # Stub
///
/// This is a stub implementation. Add the `rustworkx` dependency and
/// replace the stub logic with real graph operations.
#[derive(Debug, Clone)]
pub struct NetworkXGraph {
    /// Nodes stored as `(node_id, label)` pairs.
    pub nodes: HashMap<String, String>,
    /// Edges stored as `(source, target, label)` triples.
    pub edges: Vec<(String, String, String)>,
}

impl NetworkXGraph {
    /// Creates a new empty [`NetworkXGraph`].
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Adds a node with the given id and optional label.
    pub fn add_node(&mut self, id: &str, label: &str) -> Result<()> {
        self.nodes.insert(id.to_string(), label.to_string());
        Ok(())
    }

    /// Adds an edge between two nodes with an optional label.
    pub fn add_edge(&mut self, source: &str, target: &str, label: &str) -> Result<()> {
        self.edges
            .push((source.to_string(), target.to_string(), label.to_string()));
        Ok(())
    }

    /// Returns the shortest path between two nodes (stub).
    pub fn shortest_path(&self, _from: &str, _to: &str) -> Result<Vec<String>> {
        Err(ChainError::ToolError(
            "NetworkX shortest_path not implemented (stub)".into(),
        ))
    }

    /// Returns the number of nodes in the graph.
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges in the graph.
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }
}

impl Default for NetworkXGraph {
    fn default() -> Self {
        Self::new()
    }
}
