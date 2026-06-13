//! Gmail document loader.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::{ChainError, Result};
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

use crate::traits::BaseLoader;

pub struct GmailLoader {
    api_key: String,
    client: Client,
}

impl GmailLoader {
    pub fn new() -> Self {
        let api_key = std::env::var("GMAIL_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .expect("GMAIL_API_KEY or GOOGLE_API_KEY environment variable is required");
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    async fn api_get(&self, endpoint: &str) -> Result<String> {
        let sep = if endpoint.contains('?') { "&" } else { "?" };
        let url = format!(
            "https://gmail.googleapis.com/gmail/v1{}{}key={}",
            endpoint, sep, self.api_key
        );
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| ChainError::IOError(format!("Gmail API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Gmail API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Gmail API response: {}", e)))
    }

    pub async fn search(&self, query: &str, max_results: Option<u32>) -> Result<Vec<Document>> {
        let max = max_results.unwrap_or(50);
        let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        let endpoint = format!("/users/me/messages?q={}&maxResults={}", encoded, max);
        let body = self.api_get(&endpoint).await?;
        let value: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Gmail search results: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(messages) = value.get("messages").and_then(|m| m.as_array()) {
            for msg_ref in messages {
                if let Some(msg_id) = msg_ref.get("id").and_then(|v| v.as_str()) {
                    let doc = self.get_message(msg_id).await?;
                    documents.push(doc);
                }
            }
        }

        Ok(documents)
    }

    pub async fn get_message(&self, message_id: &str) -> Result<Document> {
        let endpoint = format!("/users/me/messages/{}?format=full", message_id);
        let body = self.api_get(&endpoint).await?;
        let msg: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Gmail message: {}", e)))?;

        let headers = msg.pointer("/payload/headers").and_then(|h| h.as_array()).cloned().unwrap_or_default();
        let mut subject = String::new();
        let mut from = String::new();
        let mut to = String::new();
        let mut date = String::new();

        for header in &headers {
            let name = header.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let value = header.get("value").and_then(|v| v.as_str()).unwrap_or("");
            match name {
                "Subject" => subject = value.to_string(),
                "From" => from = value.to_string(),
                "To" => to = value.to_string(),
                "Date" => date = value.to_string(),
                _ => {}
            }
        }

        let body_text = extract_gmail_body(&msg);
        let content = format!("Subject: {}\nFrom: {}\nTo: {}\nDate: {}\n\n{}", subject, from, to, date, body_text);

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String(format!("gmail:{}", message_id)));
        metadata.insert("message_id".to_string(), serde_json::Value::String(message_id.to_string()));
        metadata.insert("subject".to_string(), serde_json::Value::String(subject));
        metadata.insert("from".to_string(), serde_json::Value::String(from));
        metadata.insert("to".to_string(), serde_json::Value::String(to));
        metadata.insert("date".to_string(), serde_json::Value::String(date));
        metadata.insert("loader_type".to_string(), serde_json::Value::String("gmail".to_string()));

        Ok(Document::new(content).with_metadata(metadata))
    }

    pub async fn get_thread(&self, thread_id: &str) -> Result<Vec<Document>> {
        let endpoint = format!("/users/me/threads/{}?format=full", thread_id);
        let body = self.api_get(&endpoint).await?;
        let thread: Value = serde_json::from_str(&body)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Gmail thread: {}", e)))?;

        let mut documents = Vec::new();
        if let Some(messages) = thread.get("messages").and_then(|m| m.as_array()) {
            for msg in messages {
                if let Some(msg_id) = msg.get("id").and_then(|v| v.as_str()) {
                    let mut metadata = HashMap::new();
                    metadata.insert("source".to_string(), serde_json::Value::String(format!("gmail_thread:{}", thread_id)));
                    metadata.insert("thread_id".to_string(), serde_json::Value::String(thread_id.to_string()));
                    metadata.insert("message_id".to_string(), serde_json::Value::String(msg_id.to_string()));
                    metadata.insert("loader_type".to_string(), serde_json::Value::String("gmail".to_string()));

                    let body_text = extract_gmail_body(msg);
                    let headers = msg.pointer("/payload/headers").and_then(|h| h.as_array()).cloned().unwrap_or_default();
                    for header in &headers {
                        let name = header.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let value = header.get("value").and_then(|v| v.as_str()).unwrap_or("");
                        match name {
                            "Subject" => { metadata.insert("subject".to_string(), serde_json::Value::String(value.to_string())); }
                            "From" => { metadata.insert("from".to_string(), serde_json::Value::String(value.to_string())); }
                            _ => {}
                        }
                    }

                    documents.push(Document::new(body_text).with_metadata(metadata));
                }
            }
        }

        Ok(documents)
    }
}

fn extract_gmail_body(msg: &Value) -> String {
    if let Some(body) = msg.pointer("/payload/body/data").and_then(|v| v.as_str()) {
        if !body.is_empty() {
            return decode_base64_url(body);
        }
    }

    if let Some(parts) = msg.pointer("/payload/parts").and_then(|p| p.as_array()) {
        for part in parts {
            let mime = part.get("mimeType").and_then(|v| v.as_str()).unwrap_or("");
            if mime == "text/plain" {
                if let Some(data) = part.pointer("/body/data").and_then(|v| v.as_str()) {
                    if !data.is_empty() {
                        return decode_base64_url(data);
                    }
                }
            }
            if mime == "text/html" {
                if let Some(data) = part.pointer("/body/data").and_then(|v| v.as_str()) {
                    if !data.is_empty() {
                        return decode_base64_url(data);
                    }
                }
            }
            if let Some(subparts) = part.get("parts").and_then(|p| p.as_array()) {
                for subpart in subparts {
                    let mime = subpart.get("mimeType").and_then(|v| v.as_str()).unwrap_or("");
                    if mime == "text/plain" {
                        if let Some(data) = subpart.pointer("/body/data").and_then(|v| v.as_str()) {
                            if !data.is_empty() {
                                return decode_base64_url(data);
                            }
                        }
                    }
                }
            }
        }
    }

    String::new()
}

fn decode_base64_url(encoded: &str) -> String {
    let cleaned = encoded.replace('-', "+").replace('_', "/");
    let padded = match cleaned.len() % 4 {
        2 => format!("{}==", cleaned),
        3 => format!("{}=", cleaned),
        _ => cleaned,
    };
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(padded.as_bytes())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}

impl Default for GmailLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for GmailLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        self.search("in:inbox", Some(20)).await
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        match self.load().await {
            Ok(docs) => futures::stream::iter(docs.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        }
    }
}
