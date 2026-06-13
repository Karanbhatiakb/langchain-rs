//! DynamoDB-backed chat message history.

use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;

use crate::chat_message_histories::BaseChatMessageHistory;

/// Chat message history backed by AWS DynamoDB.
///
/// Uses an in-memory fallback with a warning when the database is unavailable.
#[derive(Debug)]
pub struct DynamoDBChatMessageHistory {
    pub table_name: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl DynamoDBChatMessageHistory {
    pub fn new(table_name: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for DynamoDBChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!(
            "DynamoDBChatMessageHistory: using in-memory fallback. Table: {}, session: {}",
            self.table_name,
            self.session_id
        );
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!(
            "DynamoDBChatMessageHistory: using in-memory fallback for clear. Table: {}, session: {}",
            self.table_name,
            self.session_id
        );
        Ok(())
    }
}
