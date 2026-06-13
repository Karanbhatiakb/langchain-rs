//! Sub-graph composition — embed one compiled graph as a node inside another.

use async_trait::async_trait;
use langchain_core::errors::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;

use crate::graph::CompiledGraph;
use crate::nodes::Node;
use crate::state::StateSchema;

pub struct SubGraph<S: StateSchema> {
    name: String,
    graph: Arc<CompiledGraph<S>>,
}

impl<S: StateSchema> SubGraph<S> {
    pub fn new(name: impl Into<String>, graph: CompiledGraph<S>) -> Self {
        Self {
            name: name.into(),
            graph: Arc::new(graph),
        }
    }

    pub fn with_compiled(name: impl Into<String>, graph: Arc<CompiledGraph<S>>) -> Self {
        Self {
            name: name.into(),
            graph,
        }
    }
}

#[async_trait]
impl<S: StateSchema + Serialize + DeserializeOwned> Node<S> for SubGraph<S> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, state: S) -> Result<S> {
        self.graph.invoke(state).await
    }
}

pub struct SubGraphBuilder<S: StateSchema> {
    name: String,
    graph: Option<Arc<CompiledGraph<S>>>,
}

impl<S: StateSchema> SubGraphBuilder<S> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            graph: None,
        }
    }

    pub fn with_graph(mut self, graph: CompiledGraph<S>) -> Self {
        self.graph = Some(Arc::new(graph));
        self
    }

    pub fn with_compiled_graph(mut self, graph: Arc<CompiledGraph<S>>) -> Self {
        self.graph = Some(graph);
        self
    }

    pub fn build(self) -> Result<SubGraph<S>> {
        let graph = self
            .graph
            .ok_or_else(|| langchain_core::errors::ChainError::AgentError(
                "SubGraph requires a compiled graph".to_string(),
            ))?;
        Ok(SubGraph {
            name: self.name,
            graph,
        })
    }
}
