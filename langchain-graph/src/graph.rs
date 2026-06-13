//! Graph-based execution graph state machine.

use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::errors::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;

use crate::checkpoint::Checkpointer;
use crate::edges::{ConditionalEdge, Edge, END, START};
use crate::nodes::Node;
use crate::state::StateSchema;

pub struct StateGraph<S: StateSchema> {
    nodes: HashMap<String, Arc<dyn Node<S>>>,
    edges: Vec<Edge<S>>,
    conditional_edges: Vec<ConditionalEdge<S>>,
    entry_point: Option<String>,
    finish_points: HashSet<String>,
    checkpointer: Option<Arc<dyn Checkpointer>>,
}

impl<S: StateSchema> StateGraph<S> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            conditional_edges: Vec::new(),
            entry_point: None,
            finish_points: HashSet::new(),
            checkpointer: None,
        }
    }

    pub fn with_checkpointer(mut self, checkpointer: Arc<dyn Checkpointer>) -> Self {
        self.checkpointer = Some(checkpointer);
        self
    }

    pub fn add_node(
        &mut self,
        name: impl Into<String>,
        node: Arc<dyn Node<S>>,
    ) {
        let name = name.into();
        if name == START || name == END {
            panic!("Cannot add node with reserved name: {}", name);
        }
        self.nodes.insert(name, node);
    }

    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) {
        let from = from.into();
        let to = to.into();
        self.edges.push(Edge::new(from, to));
    }

    pub fn add_conditional_edges(
        &mut self,
        from: impl Into<String>,
        condition_fn: Box<dyn Fn(&S) -> String + Send + Sync>,
        mapping: HashMap<String, String>,
    ) {
        let from = from.into();
        self.conditional_edges
            .push(ConditionalEdge::new(from, condition_fn, mapping));
    }

    pub fn set_entry_point(&mut self, name: impl Into<String>) {
        self.entry_point = Some(name.into());
    }

    pub fn set_finish_point(&mut self, name: impl Into<String>) {
        self.finish_points.insert(name.into());
    }

    pub fn compile(self) -> CompiledGraph<S> {
        let entry = self.entry_point.unwrap_or_else(|| {
            self.nodes
                .keys()
                .next()
                .cloned()
                .expect("No nodes in graph")
        });
        let finish = if self.finish_points.is_empty() {
            let mut set = HashSet::new();
            set.insert(END.to_string());
            set
        } else {
            self.finish_points
        };
        CompiledGraph {
            nodes: self.nodes,
            edges: self.edges,
            conditional_edges: self.conditional_edges,
            entry_point: entry,
            finish_points: finish,
            checkpointer: self.checkpointer,
        }
    }
}

impl<S: StateSchema> Default for StateGraph<S> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompiledGraph<S: StateSchema> {
    nodes: HashMap<String, Arc<dyn Node<S>>>,
    edges: Vec<Edge<S>>,
    conditional_edges: Vec<ConditionalEdge<S>>,
    entry_point: String,
    finish_points: HashSet<String>,
    checkpointer: Option<Arc<dyn Checkpointer>>,
}

impl<S: StateSchema> CompiledGraph<S> {
    fn get_next_nodes(&self, current: &str, state: &S) -> Vec<String> {
        let mut next = Vec::new();

        for edge in &self.edges {
            if edge.from == current {
                next.push(edge.to.clone());
            }
        }

        for cond_edge in &self.conditional_edges {
            if cond_edge.from == current {
                let target = cond_edge.evaluate(state);
                next.push(target);
            }
        }

        if next.is_empty() && !self.finish_points.contains(current) {
            next.push(END.to_string());
        }

        next
    }

    async fn execute_graph(&self, mut state: S) -> Result<S> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: Vec<String> = vec![self.entry_point.clone()];
        let max_iterations = 100;
        let mut iterations = 0;

        while let Some(current) = queue.pop() {
            if current == END {
                break;
            }

            iterations += 1;
            if iterations > max_iterations {
                return Err(ChainError::AgentError(
                    "Graph exceeded max iterations".to_string(),
                ));
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if self.finish_points.contains(&current) {
                break;
            }

            if let Some(node) = self.nodes.get(&current) {
                state = node.run(state).await?;
            }

            let next = self.get_next_nodes(&current, &state);
            for n in next {
                queue.push(n);
            }
        }

        Ok(state)
    }
}

impl<S: StateSchema + Serialize + DeserializeOwned> CompiledGraph<S> {
    pub async fn invoke(&self, input_state: S) -> Result<S> {
        if let Some(ref cp) = self.checkpointer {
            if let Some(saved_value) = cp.load("default").await? {
                let saved: S = serde_json::from_value(saved_value)
                    .map_err(|e| ChainError::ParserError(e.to_string()))?;
                info!("Loaded checkpointed state");
                return self.execute_graph(saved).await;
            }
        }

        let result = self.execute_graph(input_state).await?;

        if let Some(ref cp) = self.checkpointer {
            let value = serde_json::to_value(&result)
                .map_err(|e| ChainError::ParserError(e.to_string()))?;
            cp.save("default", &value).await?;
        }

        Ok(result)
    }

    pub fn stream(
        &self,
        input_state: S,
    ) -> BoxStream<'static, Result<S>> {
        let (tx, rx) = mpsc::channel::<Result<S>>(64);

        let nodes = self.nodes.clone();
        let edges = self.edges.clone();
        let conditional_edges: Vec<ConditionalEdge<S>> = self.conditional_edges.iter().map(|e| e.clone_box()).collect();
        let entry_point = self.entry_point.clone();
        let finish_points = self.finish_points.clone();

        tokio::spawn(async move {
            let mut state = input_state;
            let mut visited: HashSet<String> = HashSet::new();
            let mut queue: Vec<String> = vec![entry_point];
            let max_iterations = 100;
            let mut iterations = 0;

            while let Some(current) = queue.pop() {
                if current == END {
                    break;
                }

                iterations += 1;
                if iterations > max_iterations {
                    let _ = tx
                        .send(Err(ChainError::AgentError(
                            "Graph exceeded max iterations".to_string(),
                        )))
                        .await;
                    break;
                }

                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current.clone());

                if finish_points.contains(&current) {
                    break;
                }

                if let Some(node) = nodes.get(&current) {
                    match node.run(state).await {
                        Ok(new_state) => {
                            state = new_state;
                            let _ = tx.send(Ok(state.clone())).await;
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e)).await;
                            break;
                        }
                    }
                }

                let mut next = Vec::new();
                for edge in &edges {
                    if edge.from == current {
                        next.push(edge.to.clone());
                    }
                }
                for cond_edge in &conditional_edges {
                    if cond_edge.from == current {
                        let target = cond_edge.evaluate(&state);
                        next.push(target);
                    }
                }
                if next.is_empty() && !finish_points.contains(&current) {
                    next.push(END.to_string());
                }

                for n in next {
                    queue.push(n);
                }
            }
        });

        ReceiverStream::new(rx).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::NodeConfig;
    use crate::state::AgentState;

    fn identity_node(name: &str) -> Arc<dyn Node<AgentState>> {
        let name = name.to_string();
        Arc::new(NodeConfig::new(name, |state: AgentState| {
            Box::pin(async move { Ok(state) })
        }))
    }

    #[test]
    fn test_state_graph_new() {
        let g: StateGraph<AgentState> = StateGraph::new();
        assert!(g.compile().entry_point.is_empty());
    }

    #[test]
    fn test_state_graph_default() {
        let g: StateGraph<AgentState> = StateGraph::default();
        assert!(g.compile().entry_point.is_empty());
    }

    #[test]
    fn test_state_graph_add_node() {
        let mut g: StateGraph<AgentState> = StateGraph::new();
        g.add_node("a", identity_node("a"));
        let compiled = g.compile();
        assert_eq!(compiled.entry_point, "a");
    }

    #[test]
    fn test_state_graph_add_edge() {
        let mut g: StateGraph<AgentState> = StateGraph::new();
        g.add_node("a", identity_node("a"));
        g.add_node("b", identity_node("b"));
        g.add_edge("a", "b");
        g.set_entry_point("a");
        let compiled = g.compile();
        assert_eq!(compiled.entry_point, "a");
    }

    #[test]
    fn test_state_graph_set_entry_point() {
        let mut g: StateGraph<AgentState> = StateGraph::new();
        g.add_node("start", identity_node("start"));
        g.set_entry_point("start");
        let compiled = g.compile();
        assert_eq!(compiled.entry_point, "start");
    }

    #[test]
    fn test_state_graph_set_finish_point() {
        let mut g: StateGraph<AgentState> = StateGraph::new();
        g.add_node("only", identity_node("only"));
        g.set_finish_point("only");
        let compiled = g.compile();
        assert!(compiled.finish_points.contains("only"));
    }

    #[test]
    fn test_state_graph_add_conditional_edges() {
        use std::collections::HashMap;
        let mut g: StateGraph<AgentState> = StateGraph::new();
        g.add_node("check", identity_node("check"));
        g.add_node("end", identity_node("end"));
        g.add_conditional_edges(
            "check",
            Box::new(|_: &AgentState| "done".into()),
            HashMap::from([("done".into(), "end".into())]),
        );
        g.set_entry_point("check");
        g.set_finish_point("end");
        let compiled = g.compile();
        assert_eq!(compiled.conditional_edges.len(), 1);
    }

    #[tokio::test]
    async fn test_compiled_graph_execute_single_node() {
        let mut g: StateGraph<AgentState> = StateGraph::new();
        g.add_node("only", identity_node("only"));
        g.set_finish_point("only");
        let compiled = g.compile();
        let result = compiled.execute_graph(AgentState::new(vec![])).await.unwrap();
        assert!(result.messages.is_empty());
    }

    #[tokio::test]
    async fn test_compiled_graph_execute_linear() {
        let mut g: StateGraph<AgentState> = StateGraph::new();
        g.add_node("a", identity_node("a"));
        g.add_node("b", identity_node("b"));
        g.add_edge("a", "b");
        g.set_entry_point("a");
        g.set_finish_point("b");
        let compiled = g.compile();
        let result = compiled.execute_graph(AgentState::new(vec![])).await.unwrap();
        assert!(result.messages.is_empty());
    }

    #[tokio::test]
    async fn test_compiled_graph_execute_reaches_end() {
        let mut g: StateGraph<AgentState> = StateGraph::new();
        g.add_node("a", identity_node("a"));
        g.add_edge("a", END);
        g.set_entry_point("a");
        let compiled = g.compile();
        let result = compiled.execute_graph(AgentState::new(vec![])).await.unwrap();
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_state_graph_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<StateGraph<AgentState>>();
        assert_sync::<StateGraph<AgentState>>();
        assert_send::<CompiledGraph<AgentState>>();
        assert_sync::<CompiledGraph<AgentState>>();
    }
}
