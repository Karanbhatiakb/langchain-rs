use std::collections::HashMap;
use std::sync::Arc;

use langchain_core::messages::{BaseMessage, MessageType};
use langchain_graph::checkpoint::{Checkpoint, MemoryCheckpointer, Checkpointer};
use langchain_graph::edges::{Edge, END, START};
use langchain_graph::graph::StateGraph;
use langchain_graph::nodes::{LambdaNode, Node, NodeConfig};
use langchain_graph::state::{AgentState, AgentStateUpdate, MessagesState, MessagesStateUpdate, StateSchema};

#[tokio::test]
async fn test_memory_checkpointer_save_load() {
    let cp = MemoryCheckpointer::new();
    let state = serde_json::json!({"key": "value"});
    cp.save("test", &state).await.unwrap();
    let loaded = cp.load("test").await.unwrap();
    assert_eq!(loaded, Some(state));
}

#[tokio::test]
async fn test_memory_checkpointer_list() {
    let cp = MemoryCheckpointer::new();
    cp.save("a", &serde_json::json!(1)).await.unwrap();
    cp.save("b", &serde_json::json!(2)).await.unwrap();
    let list = cp.list().await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn test_memory_checkpointer_delete() {
    let cp = MemoryCheckpointer::new();
    cp.save("test", &serde_json::json!(42)).await.unwrap();
    cp.delete("test").await.unwrap();
    let loaded = cp.load("test").await.unwrap();
    assert!(loaded.is_none());
}

#[tokio::test]
async fn test_checkpoint_new() {
    let cp = Checkpoint::new(serde_json::json!({"data": 1}));
    assert!(cp.parent_id.is_none());
    assert!(cp.metadata.is_none());
}

#[tokio::test]
async fn test_checkpoint_with_parent() {
    let id = uuid::Uuid::new_v4();
    let cp = Checkpoint::new(serde_json::json!({})).with_parent(id);
    assert_eq!(cp.parent_id, Some(id));
}

#[tokio::test]
async fn test_checkpoint_with_metadata() {
    let cp = Checkpoint::new(serde_json::json!({})).with_metadata(serde_json::json!({"meta": true}));
    assert!(cp.metadata.is_some());
}

#[test]
fn test_edge_constants() {
    assert_eq!(START, "__start__");
    assert_eq!(END, "__end__");
}

#[test]
fn test_edge_new() {
    let edge: Edge<AgentState> = Edge::new("a", "b");
    assert_eq!(edge.from, "a");
    assert_eq!(edge.to, "b");
}

#[test]
fn test_agent_state_new() {
    let state = AgentState::new(vec![BaseMessage::new("hello", MessageType::Human)]);
    assert_eq!(state.messages.len(), 1);
    assert!(state.metadata.is_empty());
}

#[test]
fn test_agent_state_update() {
    let update = AgentStateUpdate::new()
        .with_messages(vec![BaseMessage::new("hi", MessageType::AI)])
        .with_metadata(HashMap::from([
            ("key".to_string(), serde_json::json!("val")),
        ]));
    assert!(update.messages.is_some());
    assert!(update.metadata.is_some());
}

#[test]
fn test_messages_state_new() {
    let state = MessagesState::new(vec![BaseMessage::new("test", MessageType::Human)]);
    assert_eq!(state.messages.len(), 1);
    assert!(state.channel.is_none());
}

#[test]
fn test_messages_state_update_with_channel() {
    let update = MessagesStateUpdate::new()
        .with_messages(vec![])
        .with_channel("chat");
    assert_eq!(update.channel, Some("chat".to_string()));
}

#[test]
fn test_state_schema_merge() {
    let mut state = AgentState::new(vec![BaseMessage::new("first", MessageType::Human)]);
    let update = AgentStateUpdate::new()
        .with_messages(vec![BaseMessage::new("second", MessageType::AI)]);
    state.merge(update);
    assert_eq!(state.messages.len(), 2);
}

#[tokio::test]
async fn test_graph_simple_two_nodes() {
    let mut graph: StateGraph<AgentState> = StateGraph::new();

    let node_a: Arc<dyn Node<AgentState>> = Arc::new(
        LambdaNode::new("a", |mut state: AgentState| Box::pin(async move {
            state.messages.push(BaseMessage::new("from_a", MessageType::System));
            Ok(state)
        }))
    );
    let node_b: Arc<dyn Node<AgentState>> = Arc::new(
        LambdaNode::new("b", |mut state: AgentState| Box::pin(async move {
            state.messages.push(BaseMessage::new("from_b", MessageType::System));
            Ok(state)
        }))
    );

    graph.add_node("a", node_a);
    graph.add_node("b", node_b);
    graph.add_edge("a", "b");
    graph.set_entry_point("a");

    let compiled = graph.compile();
    let initial = AgentState::new(vec![]);
    let result = compiled.invoke(initial).await.unwrap();
    assert_eq!(result.messages.len(), 2);
}

#[tokio::test]
async fn test_graph_with_checkpointer() {
    let cp = Arc::new(MemoryCheckpointer::new());
    let mut graph: StateGraph<AgentState> = StateGraph::new()
        .with_checkpointer(cp.clone());

    let node: Arc<dyn Node<AgentState>> = Arc::new(
        LambdaNode::new("n1", |mut state: AgentState| Box::pin(async move {
            state.messages.push(BaseMessage::new("processed", MessageType::System));
            Ok(state)
        }))
    );
    graph.add_node("n1", node);
    graph.set_entry_point("n1");

    let compiled = graph.compile();
    let initial = AgentState::new(vec![]);
    let result = compiled.invoke(initial).await.unwrap();
    assert_eq!(result.messages.len(), 1);
    assert_eq!(result.messages[0].content, "processed");
}

#[tokio::test]
async fn test_graph_node_config() {
    let node = NodeConfig::new("test_node", |mut state: AgentState| Box::pin(async move {
        state.metadata.insert("visited".to_string(), serde_json::json!(true));
        Ok(state)
    }));
    assert_eq!(node.name(), "test_node");
}
