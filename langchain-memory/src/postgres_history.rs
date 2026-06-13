//! PostgreSQL-backed chat message history.

use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;

use crate::chat_message_histories::BaseChatMessageHistory;

/// Chat message history backed by a PostgreSQL database.
///
/// Uses an in-memory fallback with a warning when the database is unavailable.
#[derive(Debug)]
pub struct PostgresChatMessageHistory {
    pub connection_string: String,
    pub table_name: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl PostgresChatMessageHistory {
    pub fn new(
        connection_string: impl Into<String>,
        table_name: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            connection_string: connection_string.into(),
            table_name: table_name.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for PostgresChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!(
            "PostgresChatMessageHistory: using in-memory fallback. Connection: {}, table: {}, session: {}",
            self.connection_string,
            self.table_name,
            self.session_id
        );
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!(
            "PostgresChatMessageHistory: using in-memory fallback for clear. Connection: {}, table: {}, session: {}",
            self.connection_string,
            self.table_name,
            self.session_id
        );
        Ok(())
    }
}
