//! Runnable graph — DAG of runnables with edges, ASCII/Mermaid rendering.
//!
//! Provides [`RunnableGraph`] which models a directed acyclic graph of
//! [`Runnable`] nodes connected by edges (including conditional edges),
//! validation (cycle detection, orphan-node detection), topological execution,
//! and visualisation helpers (`graph_ascii`, `graph_mermaid`).
//!
//! This is the Rust counterpart of `langchain_core.runnables.graph` from the
//! Python LangChain project.

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type RunFn = Arc<
    dyn Fn(
            HashMap<String, Value>,
        ) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>>> + Send>>
        + Send
        + Sync,
>;

/// A node in a [`RunnableGraph`].
///
/// Each node wraps a [`Runnable`] and carries a human-readable name for
/// visualisation purposes.
#[derive(Clone)]
pub struct GraphNode {
    /// Unique identifier for this node within the graph.
    pub id: String,
    /// The runnable this node executes.
    pub runnable: RunFn,
    /// Human-readable name used in visualisation.
    pub name: String,
}

impl std::fmt::Debug for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphNode")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

/// An edge in a [`RunnableGraph`].
///
/// Edges can be unconditional (always traversed) or conditional (traversed
/// only when a named condition is met, resolved externally).
#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// Source node id.
    pub source: String,
    /// Target node id.
    pub target: String,
    /// Optional condition label. When `Some`, the edge is only traversed when
    /// the condition named by this string is selected at runtime.
    pub condition: Option<String>,
}

/// A directed acyclic graph of [`Runnable`] nodes.
///
/// Nodes are added via [`RunnableGraph::add_node`], edges via
/// [`RunnableGraph::add_edge`] and [`RunnableGraph::add_conditional_edge`].
/// The graph can be validated ([`RunnableGraph::validate`]) and executed in
/// topological order via the [`Runnable`] trait implementation.
///
/// # Example
///
/// ```rust,ignore
/// use langchain_core::runnables::graph::RunnableGraph;
///
/// let mut graph = RunnableGraph::new();
/// graph.add_node("start", "Start", start_fn);
/// graph.add_node("process", "Process", process_fn);
/// graph.add_node("end", "End", end_fn);
/// graph.add_edge("start", "process");
/// graph.add_edge("process", "end");
/// graph.validate()?;
/// let output = graph.invoke(input).await?;
/// ```
pub struct RunnableGraph {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
}

impl RunnableGraph {
    /// Creates a new empty `RunnableGraph`.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Adds a node to the graph.
    ///
    /// Returns `Err(ChainError::ValidationError)` if a node with the same id
    /// already exists.
    pub fn add_node<F, Fut>(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        runnable: F,
    ) -> Result<()>
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        let id = id.into();
        if self.nodes.contains_key(&id) {
            return Err(ChainError::ValidationError(format!(
                "Node with id '{}' already exists",
                id
            )));
        }
        let name = name.into();
        let arc_fn: RunFn = Arc::new(move |input| Box::pin(runnable(input)));
        self.nodes.insert(
            id.clone(),
            GraphNode {
                id,
                runnable: arc_fn,
                name,
            },
        );
        Ok(())
    }

    /// Adds an unconditional edge from `source` to `target`.
    ///
    /// Returns `Err(ChainError::ValidationError)` if either node does not
    /// exist.
    pub fn add_edge(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Result<()> {
        let source = source.into();
        let target = target.into();
        if !self.nodes.contains_key(&source) {
            return Err(ChainError::ValidationError(format!(
                "Source node '{}' not in graph",
                source
            )));
        }
        if !self.nodes.contains_key(&target) {
            return Err(ChainError::ValidationError(format!(
                "Target node '{}' not in graph",
                target
            )));
        }
        self.edges.push(GraphEdge {
            source,
            target,
            condition: None,
        });
        Ok(())
    }

    /// Adds a conditional edge from `source` to `target`.
    ///
    /// The `condition` label identifies the condition under which this edge is
    /// traversed. Conditional edges are rendered differently in ASCII and
    /// Mermaid diagrams.
    pub fn add_conditional_edge(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        condition: impl Into<String>,
    ) -> Result<()> {
        let source = source.into();
        let target = target.into();
        let condition = condition.into();
        if !self.nodes.contains_key(&source) {
            return Err(ChainError::ValidationError(format!(
                "Source node '{}' not in graph",
                source
            )));
        }
        if !self.nodes.contains_key(&target) {
            return Err(ChainError::ValidationError(format!(
                "Target node '{}' not in graph",
                target
            )));
        }
        self.edges.push(GraphEdge {
            source,
            target,
            condition: Some(condition),
        });
        Ok(())
    }

    /// Validates the graph, returning an error if:
    ///
    /// - The graph contains a cycle.
    /// - There are orphan nodes (nodes with no incoming or outgoing edges,
    ///   excluding the case of a single-node graph).
    pub fn validate(&self) -> Result<()> {
        if self.nodes.is_empty() {
            return Ok(());
        }

        if self.detect_cycle() {
            return Err(ChainError::ValidationError(
                "Graph contains a cycle".into(),
            ));
        }

        if self.nodes.len() > 1 {
            let orphans = self.find_orphan_nodes();
            if !orphans.is_empty() {
                return Err(ChainError::ValidationError(format!(
                    "Orphan nodes found: {}",
                    orphans.join(", ")
                )));
            }
        }

        Ok(())
    }

    /// Returns a topological ordering of the node ids, or an error if a cycle
    /// is detected.
    fn topological_order(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<&String, usize> = self
            .nodes
            .keys()
            .map(|k| (k, 0))
            .collect();

        for edge in &self.edges {
            if let Some(deg) = in_degree.get_mut(&edge.target) {
                *deg += 1;
            }
        }

        let mut queue: VecDeque<&String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&k, _)| k)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(node_id) = queue.pop_front() {
            order.push(node_id.clone());
            for edge in &self.edges {
                if edge.source == *node_id {
                    if let Some(deg) = in_degree.get_mut(&edge.target) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(&edge.target);
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(ChainError::ValidationError(
                "Graph contains a cycle".into(),
            ));
        }

        Ok(order)
    }

    /// Returns `true` if the graph contains a cycle.
    fn detect_cycle(&self) -> bool {
        self.topological_order().is_err()
    }

    /// Returns the ids of nodes that have neither incoming nor outgoing edges.
    fn find_orphan_nodes(&self) -> Vec<String> {
        let mut connected: HashSet<&String> = HashSet::new();
        for edge in &self.edges {
            connected.insert(&edge.source);
            connected.insert(&edge.target);
        }
        self.nodes
            .keys()
            .filter(|id| !connected.contains(id))
            .cloned()
            .collect()
    }

    /// Returns a reference to the nodes in this graph.
    pub fn nodes(&self) -> &HashMap<String, GraphNode> {
        &self.nodes
    }

    /// Returns a reference to the edges in this graph.
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    /// Finds the single node that is not a target of any edge (the "start"
    /// node), or `None` if there is no unique candidate.
    pub fn first_node(&self) -> Option<&GraphNode> {
        let targets: HashSet<&String> = self.edges.iter().map(|e| &e.target).collect();
        let candidates: Vec<&GraphNode> = self
            .nodes
            .values()
            .filter(|n| !targets.contains(&n.id))
            .collect();
        if candidates.len() == 1 {
            Some(candidates[0])
        } else {
            None
        }
    }

    /// Finds the single node that is not a source of any edge (the "end"
    /// node), or `None` if there is no unique candidate.
    pub fn last_node(&self) -> Option<&GraphNode> {
        let sources: HashSet<&String> = self.edges.iter().map(|e| &e.source).collect();
        let candidates: Vec<&GraphNode> = self
            .nodes
            .values()
            .filter(|n| !sources.contains(&n.id))
            .collect();
        if candidates.len() == 1 {
            Some(candidates[0])
        } else {
            None
        }
    }
}

impl Default for RunnableGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RunnableGraph {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }
}

impl std::fmt::Debug for RunnableGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunnableGraph")
            .field("node_count", &self.nodes.len())
            .field("edge_count", &self.edges.len())
            .finish()
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnableGraph {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let order = self.topological_order()?;

        let mut data: HashMap<String, HashMap<String, Value>> = HashMap::new();
        for node_id in &order {
            let node = self.nodes.get(node_id).ok_or_else(|| {
                ChainError::ValidationError(format!(
                    "Node '{}' not found during execution",
                    node_id
                ))
            })?;

            let incoming: Vec<&GraphEdge> =
                self.edges.iter().filter(|e| e.target == *node_id).collect();

            let node_input = if incoming.is_empty() {
                input.clone()
            } else {
                let mut merged = HashMap::new();
                for edge in &incoming {
                    if let Some(upstream_data) = data.get(&edge.source) {
                        for (k, v) in upstream_data {
                            merged.insert(k.clone(), v.clone());
                        }
                    }
                }
                merged
            };

            let output = (node.runnable)(node_input).await?;
            data.insert(node_id.clone(), output);
        }

        let last = self.last_node();
        match last {
            Some(node) => Ok(data.remove(&node.id).unwrap_or_default()),
            None => {
                let last_id = order.last().ok_or_else(|| {
                    ChainError::ValidationError("Graph has no nodes".into())
                })?;
                Ok(data.remove(last_id).unwrap_or_default())
            }
        }
    }
}

/// Renders the graph as an ASCII art string.
///
/// Nodes are drawn as labelled boxes connected by lines. Unconditional edges
/// use `*` characters; conditional edges use `.` characters.
///
/// # Example output
///
/// ```text
/// +-------+
/// | Start |
/// +-------+
///     *
///     *
/// +---------+
/// | Process |
/// +---------+
///     *
///     *
/// +-----+
/// | End |
/// +-----+
/// ```
pub fn graph_ascii(graph: &RunnableGraph) -> String {
    if graph.nodes.is_empty() {
        return String::new();
    }

    let order = match graph.topological_order() {
        Ok(o) => o,
        Err(_) => return "<graph contains cycle>".into(),
    };

    let mut lines: Vec<String> = Vec::new();
    let mut first = true;

    for node_id in &order {
        let node = match graph.nodes.get(node_id) {
            Some(n) => n,
            None => continue,
        };
        let width = node.name.len().max(3);

        let border = format!("+{}+", "-".repeat(width + 2));
        let label = format!("| {} |", node.name);

        if !first {
            let outgoing: Vec<&GraphEdge> =
                graph.edges.iter().filter(|e| e.source == *node_id).collect();

            let incoming: Vec<&GraphEdge> =
                graph.edges.iter().filter(|e| e.target == *node_id).collect();

            let all_cond = !outgoing.is_empty()
                && outgoing.iter().all(|e| e.condition.is_some())
                || !incoming.is_empty()
                    && incoming.iter().all(|e| e.condition.is_some());

            let edge_char = if all_cond { '.' } else { '*' };
            lines.push(format!("    {0}{0}{0}", edge_char));
            lines.push(format!("    {0}{0}{0}", edge_char));
        }
        first = false;

        lines.push(border.clone());
        lines.push(label);
        lines.push(border);
    }

    lines.join("\n")
}

/// Renders the graph as a Mermaid flowchart diagram string.
///
/// Unconditional edges use `-->` syntax; conditional edges use `-.->` syntax
/// with a label.
///
/// # Example output
///
/// ```text
/// graph TD;
///     Start-->Process;
///     Process-. some_condition .->End;
/// ```
pub fn graph_mermaid(graph: &RunnableGraph) -> String {
    if graph.nodes.is_empty() {
        return "graph TD;\n".into();
    }

    let mut mermaid = String::from("graph TD;\n");

    for node in graph.nodes.values() {
        let safe_id = to_safe_mermaid_id(&node.id);
        mermaid.push_str(&format!(
            "    {}[\"{}\"]\n",
            safe_id, node.name
        ));
    }

    for edge in &graph.edges {
        let src = to_safe_mermaid_id(&edge.source);
        let tgt = to_safe_mermaid_id(&edge.target);
        match &edge.condition {
            Some(cond) => {
                mermaid.push_str(&format!(
                    "    {} -. \"{}\" .-> {}\n",
                    src, cond, tgt
                ));
            }
            None => {
                mermaid.push_str(&format!("    {} --> {}\n", src, tgt));
            }
        }
    }

    mermaid
}

/// Converts a string into a Mermaid-compatible node id.
///
/// Keeps `[a-zA-Z0-9_-]` characters unchanged; maps every other character to
/// `_` + lowercase hex codepoint.
fn to_safe_mermaid_id(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
            out.push_str(&format!("{:x}", ch as u32));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fn(
        label: &str,
    ) -> impl Fn(HashMap<String, Value>) -> std::future::Ready<Result<HashMap<String, Value>>>
           + Send
           + Sync
           + 'static {
        let label = label.to_string();
        move |_input: HashMap<String, Value>| {
            let mut result = HashMap::new();
            result.insert("node".into(), Value::String(label.clone()));
            std::future::ready(Ok(result))
        }
    }

    #[tokio::test]
    async fn test_add_nodes_and_edges() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        graph.add_node("b", "B", make_fn("b")).unwrap();
        graph.add_edge("a", "b").unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[tokio::test]
    async fn test_duplicate_node() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        let result = graph.add_node("a", "A2", make_fn("a2"));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_edge_node() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        let result = graph.add_edge("a", "b");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_no_cycle() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        graph.add_node("b", "B", make_fn("b")).unwrap();
        graph.add_edge("a", "b").unwrap();
        assert!(graph.validate().is_ok());
    }

    #[tokio::test]
    async fn test_validate_detects_cycle() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        graph.add_node("b", "B", make_fn("b")).unwrap();
        graph.add_node("c", "C", make_fn("c")).unwrap();
        graph.add_edge("a", "b").unwrap();
        graph.add_edge("b", "c").unwrap();
        graph.add_edge("c", "a").unwrap();
        assert!(graph.validate().is_err());
    }

    #[tokio::test]
    async fn test_validate_detects_orphans() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        graph.add_node("b", "B", make_fn("b")).unwrap();
        graph.add_node("c", "C", make_fn("c")).unwrap();
        graph.add_edge("a", "b").unwrap();
        // "c" is orphan
        assert!(graph.validate().is_err());
    }

    #[tokio::test]
    async fn test_graph_invoke_linear() {
        let echo_fn = |input: HashMap<String, Value>| {
            std::future::ready(Ok(input))
        };

        let mut graph = RunnableGraph::new();
        graph.add_node("start", "Start", echo_fn).unwrap();
        graph.add_node("end", "End", echo_fn).unwrap();
        graph.add_edge("start", "end").unwrap();

        let input = HashMap::from([("key".into(), Value::String("value".into()))]);
        let result = graph.invoke(input).await.unwrap();
        assert_eq!(result.get("key"), Some(&Value::String("value".into())));
    }

    #[test]
    fn test_graph_ascii() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "Start", make_fn("a")).unwrap();
        graph.add_node("b", "Process", make_fn("b")).unwrap();
        graph.add_node("c", "End", make_fn("c")).unwrap();
        graph.add_edge("a", "b").unwrap();
        graph.add_edge("b", "c").unwrap();

        let ascii = graph_ascii(&graph);
        assert!(ascii.contains("Start"));
        assert!(ascii.contains("Process"));
        assert!(ascii.contains("End"));
    }

    #[test]
    fn test_graph_mermaid() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "Start", make_fn("a")).unwrap();
        graph.add_node("b", "End", make_fn("b")).unwrap();
        graph.add_conditional_edge("a", "b", "ok").unwrap();

        let mermaid = graph_mermaid(&graph);
        assert!(mermaid.starts_with("graph TD;"));
        assert!(mermaid.contains("Start"));
        assert!(mermaid.contains("End"));
        assert!(mermaid.contains("-. \"ok\" .->"));
    }

    #[test]
    fn test_graph_mermaid_empty() {
        let graph = RunnableGraph::new();
        let mermaid = graph_mermaid(&graph);
        assert_eq!(mermaid, "graph TD;\n");
    }

    #[test]
    fn test_first_and_last_node() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "Start", make_fn("a")).unwrap();
        graph.add_node("b", "Middle", make_fn("b")).unwrap();
        graph.add_node("c", "End", make_fn("c")).unwrap();
        graph.add_edge("a", "b").unwrap();
        graph.add_edge("b", "c").unwrap();

        assert_eq!(graph.first_node().map(|n| n.id.as_str()), Some("a"));
        assert_eq!(graph.last_node().map(|n| n.id.as_str()), Some("c"));
    }

    #[test]
    fn test_conditional_edge_add() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        graph.add_node("b", "B", make_fn("b")).unwrap();
        graph.add_conditional_edge("a", "b", "if_ok").unwrap();
        assert_eq!(graph.edges().len(), 1);
        assert_eq!(graph.edges()[0].condition.as_deref(), Some("if_ok"));
    }

    #[test]
    fn test_conditional_edge_missing_source() {
        let mut graph = RunnableGraph::new();
        graph.add_node("b", "B", make_fn("b")).unwrap();
        let result = graph.add_conditional_edge("a", "b", "cond");
        assert!(result.is_err());
    }

    #[test]
    fn test_conditional_edge_missing_target() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        let result = graph.add_conditional_edge("a", "b", "cond");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_graph_validate() {
        let graph = RunnableGraph::new();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_single_node_graph_validate() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_first_node_ambiguous() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        graph.add_node("b", "B", make_fn("b")).unwrap();
        assert!(graph.first_node().is_none());
    }

    #[test]
    fn test_last_node_ambiguous() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        graph.add_node("b", "B", make_fn("b")).unwrap();
        assert!(graph.last_node().is_none());
    }

    #[test]
    fn test_graph_ascii_empty() {
        let graph = RunnableGraph::new();
        let ascii = graph_ascii(&graph);
        assert!(ascii.is_empty());
    }

    #[test]
    fn test_graph_mermaid_with_conditional() {
        let mut graph = RunnableGraph::new();
        graph.add_node("start", "Start", make_fn("start")).unwrap();
        graph.add_node("end", "End", make_fn("end")).unwrap();
        graph.add_conditional_edge("start", "end", "ok").unwrap();
        let mermaid = graph_mermaid(&graph);
        assert!(mermaid.contains("-. \"ok\" .->"));
    }

    #[test]
    fn test_to_safe_mermaid_id() {
        let safe = to_safe_mermaid_id("hello-world_123");
        assert_eq!(safe, "hello-world_123");
        let safe = to_safe_mermaid_id("foo.bar");
        assert_ne!(safe, "foo.bar");
        assert!(safe.contains("foo"));
    }

    #[tokio::test]
    async fn test_graph_invoke_merges_upstream() {
        let fn_a = |_input: HashMap<String, Value>| {
            std::future::ready(Ok(HashMap::from([("a_key".into(), Value::String("a_val".into()))])))
        };
        let fn_b = |input: HashMap<String, Value>| {
            let mut r = input;
            r.insert("b_key".into(), Value::String("b_val".into()));
            std::future::ready(Ok(r))
        };
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", fn_a).unwrap();
        graph.add_node("b", "B", fn_b).unwrap();
        graph.add_edge("a", "b").unwrap();
        let result = graph.invoke(HashMap::new()).await.unwrap();
        assert_eq!(result.get("a_key"), Some(&Value::String("a_val".into())));
        assert_eq!(result.get("b_key"), Some(&Value::String("b_val".into())));
    }

    #[test]
    fn test_graph_node_clone() {
        let mut graph = RunnableGraph::new();
        graph.add_node("a", "A", make_fn("a")).unwrap();
        let cloned = graph.clone();
        assert_eq!(cloned.nodes().len(), 1);
    }
}
