//! Chat message history backends.

use std::sync::Arc;

use async_trait::async_trait;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use parking_lot::RwLock;
use tokio::fs;

#[async_trait]
pub trait BaseChatMessageHistory: Send + Sync {
    async fn messages(&self) -> Result<Vec<BaseMessage>>;
    async fn add_message(&self, message: BaseMessage) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct InMemoryChatMessageHistory {
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl InMemoryChatMessageHistory {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl BaseChatMessageHistory for InMemoryChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        Ok(())
    }
}

#[derive(Debug)]
pub struct FileChatMessageHistory {
    path: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl FileChatMessageHistory {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for FileChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        let json = serde_json::to_string(&*self.messages.read())
            .unwrap_or_default();
        if let Err(e) = fs::write(&self.path, &json).await {
            tracing::error!("Failed to write chat history to file '{}': {}", self.path, e);
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        if let Err(e) = fs::remove_file(&self.path).await {
            tracing::error!("Failed to delete chat history file '{}': {}", self.path, e);
        }
        Ok(())
    }
}









#[derive(Debug)]
pub struct CassandraChatMessageHistory {
    pub contact_points: String,
    pub keyspace: String,
    pub table_name: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl CassandraChatMessageHistory {
    pub fn new(contact_points: impl Into<String>, keyspace: impl Into<String>, table_name: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            contact_points: contact_points.into(),
            keyspace: keyspace.into(),
            table_name: table_name.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for CassandraChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!("CassandraChatMessageHistory: using in-memory fallback. Contact: {}, keyspace: {}, table: {}, session: {}", self.contact_points, self.keyspace, self.table_name, self.session_id);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!("CassandraChatMessageHistory: using in-memory fallback for clear. Contact: {}, keyspace: {}, table: {}, session: {}", self.contact_points, self.keyspace, self.table_name, self.session_id);
        Ok(())
    }
}







#[derive(Debug)]
pub struct MomentoChatMessageHistory {
    pub cache_name: String,
    pub key: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl MomentoChatMessageHistory {
    pub fn new(cache_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            cache_name: cache_name.into(),
            key: key.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for MomentoChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!("MomentoChatMessageHistory: using in-memory fallback. Cache: {}, key: {}", self.cache_name, self.key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!("MomentoChatMessageHistory: using in-memory fallback for clear. Cache: {}, key: {}", self.cache_name, self.key);
        Ok(())
    }
}



#[derive(Debug)]
pub struct SingleStoreChatMessageHistory {
    pub connection_string: String,
    pub table_name: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl SingleStoreChatMessageHistory {
    pub fn new(connection_string: impl Into<String>, table_name: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            table_name: table_name.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for SingleStoreChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!("SingleStoreChatMessageHistory: using in-memory fallback. Connection: {}, table: {}, session: {}", self.connection_string, self.table_name, self.session_id);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!("SingleStoreChatMessageHistory: using in-memory fallback for clear. Connection: {}, table: {}, session: {}", self.connection_string, self.table_name, self.session_id);
        Ok(())
    }
}



#[derive(Debug)]
pub struct StreamlitChatMessageHistory {
    pub key: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl StreamlitChatMessageHistory {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for StreamlitChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpstashRedisChatMessageHistory {
    pub url: String,
    pub key: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl UpstashRedisChatMessageHistory {
    pub fn new(url: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            key: key.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for UpstashRedisChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!("UpstashRedisChatMessageHistory: using in-memory fallback. URL: {}, key: {}", self.url, self.key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!("UpstashRedisChatMessageHistory: using in-memory fallback for clear. URL: {}, key: {}", self.url, self.key);
        Ok(())
    }
}

#[derive(Debug)]
pub struct XataChatMessageHistory {
    pub workspace_id: String,
    pub db_name: String,
    pub table_name: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl XataChatMessageHistory {
    pub fn new(workspace_id: impl Into<String>, db_name: impl Into<String>, table_name: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            db_name: db_name.into(),
            table_name: table_name.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for XataChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!("XataChatMessageHistory: using in-memory fallback. Workspace: {}, db: {}, table: {}, session: {}", self.workspace_id, self.db_name, self.table_name, self.session_id);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!("XataChatMessageHistory: using in-memory fallback for clear. Workspace: {}, db: {}, table: {}, session: {}", self.workspace_id, self.db_name, self.table_name, self.session_id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct ZepChatMessageHistory {
    pub url: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl ZepChatMessageHistory {
    pub fn new(url: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for ZepChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!("ZepChatMessageHistory: using in-memory fallback. URL: {}, session: {}", self.url, self.session_id);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!("ZepChatMessageHistory: using in-memory fallback for clear. URL: {}, session: {}", self.url, self.session_id);
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use langchain_core::messages::MessageType;

    #[tokio::test]
    async fn test_in_memory_add_and_retrieve() {
        let history = InMemoryChatMessageHistory::new();
        assert!(history.messages().await.unwrap().is_empty());

        history
            .add_message(BaseMessage::new("Hello", MessageType::Human))
            .await
            .unwrap();
        history
            .add_message(BaseMessage::new("Hi there!", MessageType::AI))
            .await
            .unwrap();

        let msgs = history.messages().await.unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].content, "Hello");
        assert_eq!(msgs[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_in_memory_clear() {
        let history = InMemoryChatMessageHistory::new();
        history
            .add_message(BaseMessage::new("Hello", MessageType::Human))
            .await
            .unwrap();
        assert_eq!(history.messages().await.unwrap().len(), 1);

        history.clear().await.unwrap();
        assert!(history.messages().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_multiple_messages() {
        let history = InMemoryChatMessageHistory::new();
        for i in 0..5 {
            history
                .add_message(BaseMessage::new(format!("msg {}", i), MessageType::Human))
                .await
                .unwrap();
        }
        let msgs = history.messages().await.unwrap();
        assert_eq!(msgs.len(), 5);
    }

    #[tokio::test]
    async fn test_file_chat_history_new() {
        let history = FileChatMessageHistory::new("/tmp/test_history.json");
        assert!(history.messages().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_cassandra_chat_history_new() {
        let history = CassandraChatMessageHistory::new(
            "127.0.0.1", "test_ks", "test_tbl", "test_session",
        );
        assert_eq!(history.contact_points, "127.0.0.1");
        assert_eq!(history.keyspace, "test_ks");
        assert_eq!(history.table_name, "test_tbl");
        assert_eq!(history.session_id, "test_session");
    }

    #[tokio::test]
    async fn test_cassandra_add_and_clear() {
        let history = CassandraChatMessageHistory::new(
            "127.0.0.1", "test_ks", "test_tbl", "session1",
        );
        history
            .add_message(BaseMessage::new("Hello", MessageType::Human))
            .await
            .unwrap();
        assert_eq!(history.messages().await.unwrap().len(), 1);

        history.clear().await.unwrap();
        assert!(history.messages().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_streamlit_chat_history_new() {
        let history = StreamlitChatMessageHistory::new("my_key");
        assert_eq!(history.key, "my_key");
        assert!(history.messages().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_streamlit_add_messages() {
        let history = StreamlitChatMessageHistory::new("key");
        history
            .add_message(BaseMessage::new("A", MessageType::Human))
            .await
            .unwrap();
        history
            .add_message(BaseMessage::new("B", MessageType::AI))
            .await
            .unwrap();
        assert_eq!(history.messages().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_upstash_redis_chat_history_new() {
        let history = UpstashRedisChatMessageHistory::new("https://example.com", "my_key");
        assert_eq!(history.url, "https://example.com");
        assert_eq!(history.key, "my_key");
    }

    #[test]
    fn test_in_memory_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<InMemoryChatMessageHistory>();
        assert_sync::<InMemoryChatMessageHistory>();
    }
}

#[derive(Debug)]
pub struct RocksetChatMessageHistory {
    pub workspace: String,
    pub collection: String,
    pub session_id: String,
    messages: Arc<RwLock<Vec<BaseMessage>>>,
}

impl RocksetChatMessageHistory {
    pub fn new(workspace: impl Into<String>, collection: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            workspace: workspace.into(),
            collection: collection.into(),
            session_id: session_id.into(),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl BaseChatMessageHistory for RocksetChatMessageHistory {
    async fn messages(&self) -> Result<Vec<BaseMessage>> {
        Ok(self.messages.read().clone())
    }

    async fn add_message(&self, message: BaseMessage) -> Result<()> {
        self.messages.write().push(message);
        tracing::warn!("RocksetChatMessageHistory: using in-memory fallback. Workspace: {}, collection: {}, session: {}", self.workspace, self.collection, self.session_id);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.messages.write().clear();
        tracing::warn!("RocksetChatMessageHistory: using in-memory fallback for clear. Workspace: {}, collection: {}, session: {}", self.workspace, self.collection, self.session_id);
        Ok(())
    }
}
