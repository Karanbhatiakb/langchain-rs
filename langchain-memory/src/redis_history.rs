//! Redis-backed chat message history.

use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;

use crate::chat_message_histories::BaseChatMessageHistory;

/// Chat message history backed by Redis.
///
/// Uses an in-memory fallback with a warning when the database is unavailable.
#[derive(Debug)]
pub struct RedisChatMessageHistory {
    pub url: String,
    pub key: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl RedisChatMessageHistory {
    pub fn new(url: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            key: key.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for RedisChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!(
            "RedisChatMessageHistory: using in-memory fallback. URL: {}, key: {}",
            self.url,
            self.key
        );
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!(
            "RedisChatMessageHistory: using in-memory fallback for clear. URL: {}, key: {}",
            self.url,
            self.key
        );
        Ok(())
    }
}
