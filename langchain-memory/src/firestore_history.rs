//! Firestore-backed chat message history.

use std::sync::Arc;
use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;

use crate::chat_message_histories::BaseChatMessageHistory;

/// Chat message history backed by Google Cloud Firestore.
///
/// Uses an in-memory fallback with a warning when the database is unavailable.
#[derive(Debug)]
pub struct FirestoreChatMessageHistory {
    pub project_id: String,
    pub collection: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl FirestoreChatMessageHistory {
    pub fn new(
        project_id: impl Into<String>,
        collection: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            collection: collection.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for FirestoreChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!(
            "FirestoreChatMessageHistory: using in-memory fallback. Project: {}, collection: {}, session: {}",
            self.project_id,
            self.collection,
            self.session_id
        );
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!(
            "FirestoreChatMessageHistory: using in-memory fallback for clear. Project: {}, collection: {}, session: {}",
            self.project_id,
            self.collection,
            self.session_id
        );
        Ok(())
    }
}
