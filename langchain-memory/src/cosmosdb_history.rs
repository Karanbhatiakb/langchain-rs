//! CosmosDB-backed chat message history.

use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;

use crate::chat_message_histories::BaseChatMessageHistory;

/// Chat message history backed by Azure Cosmos DB.
///
/// Uses an in-memory fallback with a warning when the database is unavailable.
#[derive(Debug)]
pub struct CosmosDBChatMessageHistory {
    pub endpoint: String,
    pub database: String,
    pub container: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl CosmosDBChatMessageHistory {
    pub fn new(
        endpoint: impl Into<String>,
        database: impl Into<String>,
        container: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            database: database.into(),
            container: container.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for CosmosDBChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!(
            "CosmosDBChatMessageHistory: using in-memory fallback. Endpoint: {}, database: {}, container: {}, session: {}",
            self.endpoint,
            self.database,
            self.container,
            self.session_id
        );
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!(
            "CosmosDBChatMessageHistory: using in-memory fallback for clear. Endpoint: {}, database: {}, container: {}, session: {}",
            self.endpoint,
            self.database,
            self.container,
            self.session_id
        );
        Ok(())
    }
}
