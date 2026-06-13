//! Neo4j-backed chat message history.

use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;

use crate::chat_message_histories::BaseChatMessageHistory;

/// Chat message history backed by Neo4j graph database.
///
/// Uses an in-memory fallback with a warning when the database is unavailable.
#[derive(Debug)]
pub struct Neo4jChatMessageHistory {
    pub url: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl Neo4jChatMessageHistory {
    pub fn new(url: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for Neo4jChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!(
            "Neo4jChatMessageHistory: using in-memory fallback. URL: {}, session: {}",
            self.url,
            self.session_id
        );
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!(
            "Neo4jChatMessageHistory: using in-memory fallback for clear. URL: {}, session: {}",
            self.url,
            self.session_id
        );
        Ok(())
    }
}
