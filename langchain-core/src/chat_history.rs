//! Chat message history abstractions.
//!
//! Defines the [`BaseChatMessageHistory`] trait for storing and retrieving
//! chat messages, and provides [`InMemoryChatMessageHistory`] — a simple
//! in-memory implementation backed by `Arc<RwLock<Vec<BaseMessage>>>`.

use crate::messages::BaseMessage;
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

/// Trait for chat message history stores.
///
/// Implementors manage a sequence of [`BaseMessage`]s that represent a
/// conversation. Messages can be added individually or cleared entirely.
#[async_trait]
pub trait BaseChatMessageHistory: Send + Sync + 'static {
    /// Returns all messages currently stored in the history.
    async fn messages(&self) -> Vec<BaseMessage>;

    /// Appends a message to the history.
    async fn add_message(&self, message: BaseMessage);

    /// Removes all messages from the history.
    async fn clear(&self);
}

/// A simple in-memory chat message history.
///
/// Backed by `Arc<RwLock<Vec<BaseMessage>>>` so it is safe to share across
/// threads and async tasks.
#[derive(Debug)]
pub struct InMemoryChatMessageHistory {
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl InMemoryChatMessageHistory {
    /// Creates a new empty `InMemoryChatMessageHistory`.
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Returns the number of messages currently stored.
    pub fn len(&self) -> usize {
        self.messages
            .read()
            .map(|m| m.len())
            .unwrap_or(0)
    }

    /// Returns `true` if the history contains no messages.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for InMemoryChatMessageHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for InMemoryChatMessageHistory {
    fn clone(&self) -> Self {
        Self {
            messages: Arc::clone(&self.messages),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for InMemoryChatMessageHistory {
    async fn messages(&self) -> Vec<BaseMessage> {
        self.messages
            .read()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    async fn add_message(&self, message: BaseMessage) {
        if let Ok(mut msgs) = self.messages.write() {
            msgs.push(message);
        }
    }

    async fn clear(&self) {
        if let Ok(mut msgs) = self.messages.write() {
            msgs.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::MessageType;

    #[tokio::test]
    async fn test_in_memory_history_add_and_retrieve() {
        let history = InMemoryChatMessageHistory::new();
        assert!(history.is_empty());

        history
            .add_message(BaseMessage::new("Hello", MessageType::Human))
            .await;
        history
            .add_message(BaseMessage::new("Hi there!", MessageType::AI))
            .await;

        let msgs = history.messages().await;
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].content, "Hello");
        assert_eq!(msgs[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_in_memory_history_clear() {
        let history = InMemoryChatMessageHistory::new();
        history
            .add_message(BaseMessage::new("Hello", MessageType::Human))
            .await;
        assert_eq!(history.len(), 1);

        history.clear().await;
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_history_clone_shares_state() {
        let history = InMemoryChatMessageHistory::new();
        let cloned = history.clone();

        cloned
            .add_message(BaseMessage::new("shared", MessageType::Human))
            .await;

        let msgs = history.messages().await;
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "shared");
    }

    #[tokio::test]
    async fn test_in_memory_history_add_multiple_messages() {
        let history = InMemoryChatMessageHistory::new();
        for i in 0..10 {
            history
                .add_message(BaseMessage::new(format!("msg {}", i), MessageType::Human))
                .await;
        }
        let msgs = history.messages().await;
        assert_eq!(msgs.len(), 10);
    }

    #[tokio::test]
    async fn test_in_memory_history_default() {
        let history = InMemoryChatMessageHistory::default();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[tokio::test]
    async fn test_in_memory_history_len() {
        let history = InMemoryChatMessageHistory::new();
        assert_eq!(history.len(), 0);
        history
            .add_message(BaseMessage::new("a", MessageType::Human))
            .await;
        assert_eq!(history.len(), 1);
        history
            .add_message(BaseMessage::new("b", MessageType::AI))
            .await;
        assert_eq!(history.len(), 2);
    }

    #[tokio::test]
    async fn test_in_memory_history_clear_empty() {
        let history = InMemoryChatMessageHistory::new();
        history.clear().await;
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_history_arc_shared_behavior() {
        let history = InMemoryChatMessageHistory::new();
        let cloned = history.clone();

        history
            .add_message(BaseMessage::new("from original", MessageType::Human))
            .await;
        cloned
            .add_message(BaseMessage::new("from clone", MessageType::AI))
            .await;

        let msgs = history.messages().await;
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn test_in_memory_history_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<InMemoryChatMessageHistory>();
        assert_sync::<InMemoryChatMessageHistory>();
    }
}
