//! Loading chat sessions from various formats.
//!
//! Provides the [`ChatLoader`] trait and implementations for JSON
//! ([`JsonChatLoader`]) and CSV ([`CsvChatLoader`]) file formats.

use crate::errors::*;
use crate::messages::BaseMessage;
use crate::messages::MessageType;
use async_trait::async_trait;
use std::fs;

/// Trait for loading chat sessions from a data source.
///
/// Each session is a `Vec<BaseMessage>`, and the loader returns a list of
/// sessions.
#[async_trait]
pub trait ChatLoader: Send + Sync {
    /// Loads all chat sessions from the source.
    async fn load(&self) -> Result<Vec<Vec<BaseMessage>>>;
}

/// A chat loader that reads sessions from a JSON file.
///
/// The JSON file should contain an array of arrays of [`BaseMessage`] objects.
#[derive(Debug, Clone)]
pub struct JsonChatLoader {
    /// Path to the JSON file.
    pub path: String,
}

impl JsonChatLoader {
    /// Creates a new `JsonChatLoader` for the given file path.
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait]
impl ChatLoader for JsonChatLoader {
    async fn load(&self) -> Result<Vec<Vec<BaseMessage>>> {
        let data = fs::read_to_string(&self.path)?;
        let sessions: Vec<Vec<BaseMessage>> = serde_json::from_str(&data)?;
        Ok(sessions)
    }
}

/// A chat loader that reads messages from a CSV file.
///
/// Each row in the CSV is converted to a single-message "session" based on the
/// specified content and role columns.
#[derive(Debug, Clone)]
pub struct CsvChatLoader {
    /// Path to the CSV file.
    pub path: String,
    /// Column name containing the message content.
    pub content_column: String,
    /// Column name containing the message role (e.g. "Human", "AI").
    pub role_column: String,
}

impl CsvChatLoader {
    /// Creates a new `CsvChatLoader`.
    pub fn new(
        path: impl Into<String>,
        content_column: impl Into<String>,
        role_column: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            content_column: content_column.into(),
            role_column: role_column.into(),
        }
    }

    fn parse_role(role: &str) -> MessageType {
        match role.to_lowercase().as_str() {
            "human" | "user" => MessageType::Human,
            "ai" | "assistant" => MessageType::AI,
            "system" => MessageType::System,
            "tool" => MessageType::Tool,
            "function" => MessageType::Function,
            _ => MessageType::Generic,
        }
    }
}

#[async_trait]
impl ChatLoader for CsvChatLoader {
    async fn load(&self) -> Result<Vec<Vec<BaseMessage>>> {
        let data = fs::read_to_string(&self.path)?;
        let mut rdr = csv::Reader::from_reader(data.as_bytes());
        let headers = rdr
            .headers()
            .map_err(|e| ChainError::IOError(e.to_string()))?
            .clone();

        let content_idx = headers
            .iter()
            .position(|h| h == self.content_column)
            .ok_or_else(|| {
                ChainError::ValidationError(format!(
                    "Column '{}' not found in CSV headers",
                    self.content_column
                ))
            })?;
        let role_idx = headers
            .iter()
            .position(|h| h == self.role_column)
            .ok_or_else(|| {
                ChainError::ValidationError(format!(
                    "Column '{}' not found in CSV headers",
                    self.role_column
                ))
            })?;

        let mut sessions: Vec<Vec<BaseMessage>> = Vec::new();
    for result in rdr.records() {
                let record = result.map_err(|e| ChainError::IOError(e.to_string()))?;
                let content = record.get(content_idx).unwrap_or("").to_string();
                let role = record.get(role_idx).unwrap_or("human").to_string();
                let msg = BaseMessage::new(content, Self::parse_role(&role));
                sessions.push(vec![msg]);
            }
            Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_chat_loader_stub() {
        let loader = JsonChatLoader::new("/nonexistent/path/chat.json");
        let result = loader.load().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_csv_chat_loader_stub() {
        let loader = CsvChatLoader::new("/nonexistent/path/chat.csv", "content", "role");
        let result = loader.load().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_json_chat_loader_new() {
        let loader = JsonChatLoader::new("/tmp/test.json");
        assert_eq!(loader.path, "/tmp/test.json");
    }

    #[tokio::test]
    async fn test_csv_chat_loader_new() {
        let loader = CsvChatLoader::new("/tmp/test.csv", "content", "role");
        assert_eq!(loader.path, "/tmp/test.csv");
        assert_eq!(loader.content_column, "content");
        assert_eq!(loader.role_column, "role");
    }

    #[tokio::test]
    async fn test_csv_parse_role() {
        assert!(matches!(CsvChatLoader::parse_role("human"), MessageType::Human));
        assert!(matches!(CsvChatLoader::parse_role("user"), MessageType::Human));
        assert!(matches!(CsvChatLoader::parse_role("ai"), MessageType::AI));
        assert!(matches!(CsvChatLoader::parse_role("assistant"), MessageType::AI));
        assert!(matches!(CsvChatLoader::parse_role("system"), MessageType::System));
        assert!(matches!(CsvChatLoader::parse_role("tool"), MessageType::Tool));
        assert!(matches!(CsvChatLoader::parse_role("function"), MessageType::Function));
        assert!(matches!(CsvChatLoader::parse_role("unknown"), MessageType::Generic));
    }

    #[tokio::test]
    async fn test_csv_parse_role_case_insensitive() {
        assert!(matches!(CsvChatLoader::parse_role("Human"), MessageType::Human));
        assert!(matches!(CsvChatLoader::parse_role("AI"), MessageType::AI));
    }

    #[tokio::test]
    async fn test_chat_loader_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<JsonChatLoader>();
        assert_sync::<JsonChatLoader>();
        assert_send::<CsvChatLoader>();
        assert_sync::<CsvChatLoader>();
    }
}
