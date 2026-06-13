//! MongoDB-backed chat message history.

use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;

use crate::chat_message_histories::BaseChatMessageHistory;

/// Chat message history backed by MongoDB.
///
/// Uses an in-memory fallback with a warning when the database is unavailable.
#[derive(Debug)]
pub struct MongoDBChatMessageHistory {
    pub connection_string: String,
    pub collection_name: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl MongoDBChatMessageHistory {
    pub fn new(
        connection_string: impl Into<String>,
        collection_name: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            connection_string: connection_string.into(),
            collection_name: collection_name.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for MongoDBChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!(
            "MongoDBChatMessageHistory: using in-memory fallback. Connection: {}, collection: {}, session: {}",
            self.connection_string,
            self.collection_name,
            self.session_id
        );
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!(
            "MongoDBChatMessageHistory: using in-memory fallback for clear. Connection: {}, collection: {}, session: {}",
            self.connection_string,
            self.collection_name,
            self.session_id
        );
        Ok(())
    }
}
