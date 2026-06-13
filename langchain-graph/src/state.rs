//! State schema trait and built-in state types for graph-based workflows.

use async_trait::async_trait;
use langchain_core::messages::BaseMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Trait for graph state that can be updated with typed deltas.
///
/// # Type parameters
/// * `Update` — The delta type used to merge changes into the state.
#[async_trait]
pub trait StateSchema: Clone + Send + Sync + 'static {
    /// The delta/update type for modifying state.
    type Update: Clone + Send + Sync;

    /// Merges an update into the current state.
    fn merge(&mut self, update: Self::Update);
}

/// A graph state that tracks messages and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// The list of messages in the conversation.
    pub messages: Vec<BaseMessage>,
    /// Arbitrary metadata key-value pairs.
    pub metadata: HashMap<String, Value>,
}

impl AgentState {
    /// Creates a new `AgentState` with the given messages.
    pub fn new(messages: Vec<BaseMessage>) -> Self {
        Self {
            messages,
            metadata: HashMap::new(),
        }
    }
}

#[async_trait]
impl StateSchema for AgentState {
    type Update = AgentStateUpdate;

    fn merge(&mut self, update: Self::Update) {
        if let Some(messages) = update.messages {
            self.messages.extend(messages);
        }
        if let Some(metadata) = update.metadata {
            self.metadata.extend(metadata);
        }
    }
}

/// A delta update for [`AgentState`].
#[derive(Debug, Clone)]
pub struct AgentStateUpdate {
    /// Optional messages to append.
    pub messages: Option<Vec<BaseMessage>>,
    /// Optional metadata entries to merge.
    pub metadata: Option<HashMap<String, Value>>,
}

impl AgentStateUpdate {
    /// Creates a new empty `AgentStateUpdate`.
    pub fn new() -> Self {
        Self {
            messages: None,
            metadata: None,
        }
    }

    /// Sets messages to append (builder pattern).
    pub fn with_messages(mut self, msgs: Vec<BaseMessage>) -> Self {
        self.messages = Some(msgs);
        self
    }

    /// Sets metadata to merge (builder pattern).
    pub fn with_metadata(mut self, meta: HashMap<String, Value>) -> Self {
        self.metadata = Some(meta);
        self
    }
}

impl Default for AgentStateUpdate {
    fn default() -> Self {
        Self::new()
    }
}

/// A graph state that tracks messages and an optional channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesState {
    /// The list of messages in the conversation.
    pub messages: Vec<BaseMessage>,
    /// Arbitrary metadata.
    pub metadata: HashMap<String, Value>,
    /// An optional channel identifier.
    pub channel: Option<String>,
}

impl MessagesState {
    /// Creates a new `MessagesState` with the given messages.
    pub fn new(messages: Vec<BaseMessage>) -> Self {
        Self {
            messages,
            metadata: HashMap::new(),
            channel: None,
        }
    }
}

#[async_trait]
impl StateSchema for MessagesState {
    type Update = MessagesStateUpdate;

    fn merge(&mut self, update: Self::Update) {
        if let Some(messages) = update.messages {
            self.messages.extend(messages);
        }
        if let Some(metadata) = update.metadata {
            self.metadata.extend(metadata);
        }
        if let Some(channel) = update.channel {
            self.channel = Some(channel);
        }
    }
}

/// A delta update for [`MessagesState`].
#[derive(Debug, Clone)]
pub struct MessagesStateUpdate {
    /// Optional messages to append.
    pub messages: Option<Vec<BaseMessage>>,
    /// Optional metadata entries to merge.
    pub metadata: Option<HashMap<String, Value>>,
    /// Optional channel to set.
    pub channel: Option<String>,
}

impl MessagesStateUpdate {
    /// Creates a new empty `MessagesStateUpdate`.
    pub fn new() -> Self {
        Self {
            messages: None,
            metadata: None,
            channel: None,
        }
    }

    /// Sets messages to append (builder pattern).
    pub fn with_messages(mut self, msgs: Vec<BaseMessage>) -> Self {
        self.messages = Some(msgs);
        self
    }

    /// Sets metadata to merge (builder pattern).
    pub fn with_metadata(mut self, meta: HashMap<String, Value>) -> Self {
        self.metadata = Some(meta);
        self
    }

    /// Sets the channel (builder pattern).
    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }
}

impl Default for MessagesStateUpdate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use langchain_core::messages::HumanMessage;

    #[test]
    fn test_agent_state_new() {
        let state = AgentState::new(vec![]);
        assert!(state.messages.is_empty());
        assert!(state.metadata.is_empty());
    }

    #[test]
    fn test_agent_state_new_with_messages() {
        let msg = HumanMessage::new("hello");
        let state = AgentState::new(vec![msg.into()]);
        assert_eq!(state.messages.len(), 1);
    }

    #[test]
    fn test_agent_state_merge_messages() {
        let mut state = AgentState::new(vec![]);
        let update = AgentStateUpdate::new()
            .with_messages(vec![HumanMessage::new("hi").into()]);
        state.merge(update);
        assert_eq!(state.messages.len(), 1);
    }

    #[test]
    fn test_agent_state_merge_metadata() {
        let mut state = AgentState::new(vec![]);
        let mut meta = HashMap::new();
        meta.insert("key".into(), Value::String("val".into()));
        let update = AgentStateUpdate::new().with_metadata(meta);
        state.merge(update);
        assert_eq!(state.metadata.get("key").and_then(|v| v.as_str()), Some("val"));
    }

    #[test]
    fn test_agent_state_update_default() {
        let update = AgentStateUpdate::default();
        assert!(update.messages.is_none());
        assert!(update.metadata.is_none());
    }

    #[test]
    fn test_messages_state_new() {
        let state = MessagesState::new(vec![]);
        assert!(state.channel.is_none());
    }

    #[test]
    fn test_messages_state_merge_channel() {
        let mut state = MessagesState::new(vec![]);
        let update = MessagesStateUpdate::new().with_channel("ch1");
        state.merge(update);
        assert_eq!(state.channel.as_deref(), Some("ch1"));
    }

    #[test]
    fn test_messages_state_update_default() {
        let update = MessagesStateUpdate::default();
        assert!(update.messages.is_none());
        assert!(update.channel.is_none());
    }

    #[test]
    fn test_agent_state_clone() {
        let state = AgentState::new(vec![HumanMessage::new("x").into()]);
        let cloned = state.clone();
        assert_eq!(cloned.messages.len(), 1);
    }

    #[test]
    fn test_state_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AgentState>();
        assert_sync::<AgentState>();
        assert_send::<MessagesState>();
        assert_sync::<MessagesState>();
    }
}
