//! Node abstractions for graph-based workflows.

use async_trait::async_trait;
use langchain_core::errors::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::state::StateSchema;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// Trait for graph nodes that can be named and executed with a state.
#[async_trait]
pub trait Node<S: StateSchema>: Send + Sync {
    /// Returns the node's name.
    fn name(&self) -> &str;
    /// Runs the node with the given state and returns the updated state.
    async fn run(&self, state: S) -> Result<S>;
}

/// A configurable node that runs a closure.
pub struct NodeConfig<S: StateSchema> {
    /// The node's name.
    pub name: String,
    /// The closure implementing the node's logic.
    pub runnable: Box<dyn Fn(S) -> BoxFuture<'static, Result<S>> + Send + Sync>,
    /// Optional metadata attached to the node.
    pub metadata: HashMap<String, Value>,
}

impl<S: StateSchema> NodeConfig<S> {
    /// Creates a new `NodeConfig`.
    pub fn new(
        name: impl Into<String>,
        runnable: impl Fn(S) -> BoxFuture<'static, Result<S>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            runnable: Box::new(runnable),
            metadata: HashMap::new(),
        }
    }

    /// Sets metadata (builder pattern).
    pub fn with_metadata(mut self, meta: HashMap<String, Value>) -> Self {
        self.metadata = meta;
        self
    }
}

#[async_trait]
impl<S: StateSchema> Node<S> for NodeConfig<S> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, state: S) -> Result<S> {
        (self.runnable)(state).await
    }
}

/// A simple node wrapping an async function via `Arc`.
pub struct LambdaNode<S: StateSchema> {
    name: String,
    func: Arc<dyn Fn(S) -> BoxFuture<'static, Result<S>> + Send + Sync>,
    metadata: HashMap<String, Value>,
}

impl<S: StateSchema> LambdaNode<S> {
    /// Creates a new `LambdaNode`.
    pub fn new(
        name: impl Into<String>,
        func: impl Fn(S) -> BoxFuture<'static, Result<S>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            func: Arc::new(func),
            metadata: HashMap::new(),
        }
    }

    /// Sets metadata (builder pattern).
    pub fn with_metadata(mut self, meta: HashMap<String, Value>) -> Self {
        self.metadata = meta;
        self
    }
}

#[async_trait]
impl<S: StateSchema> Node<S> for LambdaNode<S> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, state: S) -> Result<S> {
        (self.func)(state).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AgentState;

    #[tokio::test]
    async fn test_node_config_basic() {
        let node = NodeConfig::new("test", |state: AgentState| {
            Box::pin(async move { Ok(state) })
        });
        assert_eq!(node.name(), "test");
        let result = node.run(AgentState::new(vec![])).await.unwrap();
        assert!(result.messages.is_empty());
    }

    #[tokio::test]
    async fn test_node_config_with_metadata() {
        let mut meta = HashMap::new();
        meta.insert("key".into(), Value::String("val".into()));
        let node = NodeConfig::new("meta", |state: AgentState| {
            Box::pin(async move { Ok(state) })
        }).with_metadata(meta);
        assert_eq!(node.metadata.get("key").and_then(|v| v.as_str()), Some("val"));
    }

    #[tokio::test]
    async fn test_lambda_node_basic() {
        let node = LambdaNode::new("lambda", |state: AgentState| {
            Box::pin(async move { Ok(state) })
        });
        assert_eq!(node.name(), "lambda");
        let result = node.run(AgentState::new(vec![])).await.unwrap();
        assert!(result.messages.is_empty());
    }

    #[tokio::test]
    async fn test_lambda_node_transforms_state() {
        use langchain_core::messages::HumanMessage;
        let node = LambdaNode::new("add_msg", |mut state: AgentState| {
            Box::pin(async move {
                state.messages.push(HumanMessage::new("added").into());
                Ok(state)
            })
        });
        let result = node.run(AgentState::new(vec![])).await.unwrap();
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_lambda_node_with_metadata() {
        let mut meta = HashMap::new();
        meta.insert("type".into(), Value::String("lambda".into()));
        let node = LambdaNode::new("lambda", |state: AgentState| {
            Box::pin(async move { Ok(state) })
        }).with_metadata(meta);
        assert_eq!(node.metadata.get("type").and_then(|v| v.as_str()), Some("lambda"));
    }

    #[test]
    fn test_node_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<NodeConfig<AgentState>>();
        assert_sync::<NodeConfig<AgentState>>();
        assert_send::<LambdaNode<AgentState>>();
        assert_sync::<LambdaNode<AgentState>>();
    }
}
