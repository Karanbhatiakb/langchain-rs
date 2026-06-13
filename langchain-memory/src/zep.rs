//! Zep memory store implementation.

use std::collections::HashMap;
use async_trait::async_trait;
use langchain_core::errors::{ChainError, Result};
use langchain_core::messages::BaseMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;

use crate::traits::BaseMemory;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ZepSession {
    pub id: Option<String>,
    pub session_id: String,
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZepMemoryResponse {
    pub messages: Vec<ZepMessage>,
    pub summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZepMessage {
    pub role: String,
    pub content: String,
    pub metadata: Option<HashMap<String, Value>>,
}

pub struct ZepMemory {
    api_url: String,
    api_key: String,
    session_id: String,
    client: Client,
    memory_key: String,
    input_key: String,
    output_key: String,
    return_messages: bool,
}

impl ZepMemory {
    pub fn new(session_id: impl Into<String>) -> Self {
        let api_url = std::env::var("ZEP_API_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());
        let api_key = std::env::var("ZEP_API_KEY").unwrap_or_default();

        Self {
            api_url: api_url.trim_end_matches('/').to_string(),
            api_key,
            session_id: session_id.into(),
            client: Client::new(),
            memory_key: "history".to_string(),
            input_key: "input".to_string(),
            output_key: "output".to_string(),
            return_messages: false,
        }
    }

    pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = key.into();
        self
    }

    pub fn with_memory_key(mut self, key: impl Into<String>) -> Self {
        self.memory_key = key.into();
        self
    }

    pub fn with_input_key(mut self, key: impl Into<String>) -> Self {
        self.input_key = key.into();
        self
    }

    pub fn with_output_key(mut self, key: impl Into<String>) -> Self {
        self.output_key = key.into();
        self
    }

    pub fn with_return_messages(mut self, value: bool) -> Self {
        self.return_messages = value;
        self
    }

    async fn api_get(&self, endpoint: &str) -> Result<String> {
        let url = format!("{}{}", self.api_url, endpoint);
        let mut req = self.client.get(&url);

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = req.send().await
            .map_err(|e| ChainError::IOError(format!("Zep API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Zep API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Zep API response: {}", e)))
    }

    async fn api_post(&self, endpoint: &str, body: &Value) -> Result<String> {
        let url = format!("{}{}", self.api_url, endpoint);
        let mut req = self.client.post(&url).json(body);

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = req.send().await
            .map_err(|e| ChainError::IOError(format!("Zep API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Zep API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        response.text().await
            .map_err(|e| ChainError::IOError(format!("Failed to read Zep API response: {}", e)))
    }

    async fn api_delete(&self, endpoint: &str) -> Result<()> {
        let url = format!("{}{}", self.api_url, endpoint);
        let mut req = self.client.delete(&url);

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = req.send().await
            .map_err(|e| ChainError::IOError(format!("Zep API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ChainError::IOError(format!(
                "Zep API returned HTTP {} for '{}'", response.status(), endpoint
            )));
        }

        Ok(())
    }

    pub async fn add_session(&self) -> Result<()> {
        let body = serde_json::json!({
            "session_id": self.session_id,
        });
        self.api_post("/api/v1/sessions", &body).await?;
        Ok(())
    }

    pub async fn add_memory(&self, messages: Vec<ZepMessage>) -> Result<()> {
        let body = serde_json::json!({
            "messages": messages,
        });
        self.api_post(&format!("/api/v1/sessions/{}/memory", self.session_id), &body).await?;
        Ok(())
    }

    pub async fn get_memory(&self) -> Result<Option<ZepMemoryResponse>> {
        let result = self.api_get(&format!("/api/v1/sessions/{}/memory", self.session_id)).await;

        match result {
            Ok(body) => {
                let memory: ZepMemoryResponse = serde_json::from_str(&body)
                    .map_err(|e| ChainError::ParserError(format!("Failed to parse Zep memory: {}", e)))?;
                Ok(Some(memory))
            }
            Err(e) => {
                if e.to_string().contains("404") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub async fn search_memory(&self, query: &str, limit: Option<u32>) -> Result<Vec<Value>> {
        let limit = limit.unwrap_or(10);
        let body = serde_json::json!({
            "text": query,
            "limit": limit,
        });
        let result = self.api_post(&format!("/api/v1/sessions/{}/search", self.session_id), &body).await?;
        let summaries: Vec<Value> = serde_json::from_str(&result)
            .map_err(|e| ChainError::ParserError(format!("Failed to parse Zep search results: {}", e)))?;
        Ok(summaries)
    }
}

#[async_trait]
impl BaseMemory for ZepMemory {
    fn memory_variables(&self) -> Vec<String> {
        vec![self.memory_key.clone()]
    }

    async fn load_memory_variables(&self, _inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();

        match self.get_memory().await? {
            Some(memory) => {
                if self.return_messages {
                    let msgs: Vec<Value> = memory.messages.iter()
                        .map(|m| {
                            let msg_type = match m.role.as_str() {
                                "human" => langchain_core::messages::MessageType::Human,
                                "ai" => langchain_core::messages::MessageType::AI,
                                "system" => langchain_core::messages::MessageType::System,
                                _ => langchain_core::messages::MessageType::Generic,
                            };
                            let msg = BaseMessage::new(m.content.clone(), msg_type);
                            serde_json::to_value(msg).unwrap_or_default()
                        })
                        .collect();
                    result.insert(self.memory_key.clone(), Value::Array(msgs));
                } else {
                    let text = memory.messages.iter()
                        .map(|m| format!("{}: {}", m.role, m.content))
                        .collect::<Vec<_>>()
                        .join("\n");
                    result.insert(self.memory_key.clone(), Value::String(text));
                }
            }
            None => {
                result.insert(self.memory_key.clone(), Value::String(String::new()));
            }
        }

        Ok(result)
    }

    async fn save_context(&self, inputs: &HashMap<String, Value>, outputs: &HashMap<String, Value>) -> Result<()> {
        let input = inputs.get(&self.input_key)
            .or_else(|| inputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));

        let output = outputs.get(&self.output_key)
            .or_else(|| outputs.values().next())
            .cloned()
            .unwrap_or(Value::String("".to_string()));

        let input_str = input.as_str().unwrap_or("").to_string();
        let output_str = output.as_str().unwrap_or("").to_string();

        let message = ZepMessage {
            role: "human".to_string(),
            content: input_str,
            metadata: None,
        };

        let response = ZepMessage {
            role: "ai".to_string(),
            content: output_str,
            metadata: None,
        };

        let messages = vec![message, response];
        self.add_memory(messages).await?;

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.api_delete(&format!("/api/v1/sessions/{}/memory", self.session_id)).await
    }
}
