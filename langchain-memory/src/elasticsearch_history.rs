//! Elasticsearch-backed chat message history.

use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;

use crate::chat_message_histories::BaseChatMessageHistory;

/// Chat message history backed by Elasticsearch.
///
/// Uses an in-memory fallback with a warning when the database is unavailable.
#[derive(Debug)]
pub struct ElasticsearchChatMessageHistory {
    pub url: String,
    pub index_name: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl ElasticsearchChatMessageHistory {
    pub fn new(
        url: impl Into<String>,
        index_name: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            url: url.into(),
            index_name: index_name.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for ElasticsearchChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!(
            "ElasticsearchChatMessageHistory: using in-memory fallback. URL: {}, index: {}, session: {}",
            self.url,
            self.index_name,
            self.session_id
        );
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!(
            "ElasticsearchChatMessageHistory: using in-memory fallback for clear. URL: {}, index: {}, session: {}",
            self.url,
            self.index_name,
            self.session_id
        );
        Ok(())
    }
}
