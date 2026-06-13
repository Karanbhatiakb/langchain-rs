//! Graph-based state machine for building agent workflows and multi-step
//! pipelines.
//!
//! Provides the [`StateSchema`](state::StateSchema) trait, [`Node`](nodes::Node)
//! and [`Edge`](edges::Edge) abstractions, a [`StateGraph`](graph::StateGraph) runtime,
//! and checkpointing/persistence support.

pub mod state;
pub mod nodes;
pub mod edges;
pub mod graph;
pub mod checkpoint;
pub mod persistence;
pub mod tool_nodes;
pub mod subgraph;
pub mod persistence_sqlite;
pub mod persistence_postgres;
pub mod human_in_loop;
#[cfg(feature = "neo4j")]
pub mod neo4j_graph;
#[cfg(feature = "neptune")]
pub mod neptune_graph;
#[cfg(feature = "networkx")]
pub mod networkx_graph;
#[cfg(feature = "kuzu")]
pub mod kuzu_graph;
#[cfg(feature = "nebula")]
pub mod nebula_graph;
