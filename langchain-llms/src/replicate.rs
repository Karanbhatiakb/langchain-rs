//! Replicate LLM provider implementation.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::{ChainError, Result};
use langchain_core::messages::{BaseMessage, MessageType};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::traits::{BaseLLM, ChatModel, FunctionDefinition, ToolDefinition};
use crate::types::{Generation, GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

const REPLICATE_BASE_URL: &str = "https://api.replicate.com/v1";

pub struct ChatReplicate {
    model: String,
    api_key: String,
    base_url: String,
    version: Option<String>,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatReplicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatReplicate")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct ReplicatePredictionRequest {
    input: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Deserialize)]
struct ReplicatePredictionResponse {
    id: String,
    status: String,
    output: Option<serde_json::Value>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ReplicateStreamEvent {
    id: String,
    status: String,
    #[serde(default)]
    output: Option<Value>,
    error: Option<String>,
}

impl ChatReplicate {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            base_url: REPLICATE_BASE_URL.to_string(),
            version: None,
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    fn convert_messages(&self, messages: &[BaseMessage]) -> Vec<HashMap<String, Value>> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.message_type {
                    MessageType::Human => "user",
                    MessageType::AI => "assistant",
                    MessageType::System => "system",
                    _ => "user",
                };
                let mut map = HashMap::new();
                map.insert("role".to_string(), Value::String(role.to_string()));
                map.insert("content".to_string(), Value::String(msg.content.clone()));
                map
            })
            .collect()
    }

    async fn send_request(&self, request: ReplicatePredictionRequest) -> Result<ReplicatePredictionResponse> {
        let url = format!("{}/predictions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "wait")
            .json(&request)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Replicate request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Replicate API error ({}): {}",
                status, body
            )));
        }

        Ok(response.json::<ReplicatePredictionResponse>().await?)
    }

    fn model_identifier(&self) -> String {
        match &self.version {
            Some(version) => format!("{}:{}", self.model, version),
            None => self.model.clone(),
        }
    }

    async fn stream_request(
        &self,
        request: ReplicatePredictionRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!("{}/predictions", self.base_url);
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .header("Accept", "text/event-stream")
                .json(&request)
                .timeout(Duration::from_secs(300))
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Replicate stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChainError::LLMError(format!(
                    "Replicate API streaming error ({}): {}",
                    status, body
                )))?;
            }

            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut last_output_len: usize = 0;

            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk.map_err(|e| ChainError::LLMError(format!("Stream error: {}", e)))?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line.is_empty() || line.starts_with("event:") {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(event) = serde_json::from_str::<ReplicateStreamEvent>(data) {
                            if event.status == "succeeded" || event.status == "failed" {
                                break;
                            }
                            if let Some(output) = event.output {
                                if let Some(arr) = output.as_array() {
                                    if arr.len() > last_output_len {
                                        for val in &arr[last_output_len..] {
                                            if let Some(text) = val.as_str() {
                                                yield GenerationChunk::new(text.to_string());
                                            }
                                        }
                                        last_output_len = arr.len();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    fn messages_to_prompt(&self, messages: &[BaseMessage]) -> String {
        messages
            .iter()
            .map(|m| {
                let role = match m.message_type {
                    MessageType::Human => "user",
                    MessageType::AI => "assistant",
                    MessageType::System => "system",
                    _ => "user",
                };
                format!("<|{}|>\n{}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Clone for ChatReplicate {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            version: self.version.clone(),
            config: self.config.clone(),
            client: Client::new(),
            callbacks: self.callbacks.clone(),
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for ChatReplicate {
    async fn generate(&self, prompts: &[String], stop: Option<&[&str]>) -> Result<LLMResult> {
        let messages: Vec<BaseMessage> = prompts
            .iter()
            .map(|p| BaseMessage::new(p.clone(), MessageType::Human))
            .collect();
        let msg = self.predict_messages(&messages, None, stop).await?;
        Ok(LLMResult {
            generations: vec![vec![Generation {
                text: msg.content.clone(),
                message: Some(msg),
                generation_info: None,
            }]],
            llm_output: None,
        })
    }

    async fn stream(
        &self,
        prompts: &[String],
        _stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let prompt = prompts.join("\n");
        let mut input = HashMap::new();
        input.insert("prompt".to_string(), Value::String(prompt));
        if let Some(t) = self.config.temperature {
            input.insert("temperature".to_string(), Value::from(t));
        }
        if let Some(m) = self.config.max_tokens {
            input.insert("max_tokens".to_string(), Value::from(m));
        }
        let request = ReplicatePredictionRequest {
            input,
            stream: Some(true),
        };
        self.stream_request(request).await
    }

    fn with_config(&self, config: GenerationConfig) -> Arc<dyn BaseLLM> {
        let mut new = self.clone();
        new.config = config;
        Arc::new(new)
    }

    fn with_callbacks(&self, callbacks: CallbackManager) -> Arc<dyn BaseLLM> {
        let mut new = self.clone();
        new.callbacks = callbacks;
        Arc::new(new)
    }
}

#[async_trait]
impl ChatModel for ChatReplicate {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        _stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let prompt = self.messages_to_prompt(messages);
        let mut input = HashMap::new();
        input.insert("prompt".to_string(), Value::String(prompt));
        if let Some(t) = self.config.temperature {
            input.insert("temperature".to_string(), Value::from(t));
        }
        if let Some(m) = self.config.max_tokens {
            input.insert("max_tokens".to_string(), Value::from(m));
        }
        let request = ReplicatePredictionRequest {
            input,
            stream: None,
        };
        let response = self.send_request(request).await?;
        let text = match response.output {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
                .join(""),
            Some(Value::String(s)) => s,
            _ => String::new(),
        };
        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        _stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let prompt = self.messages_to_prompt(messages);
        let mut input = HashMap::new();
        input.insert("prompt".to_string(), Value::String(prompt));
        if let Some(t) = self.config.temperature {
            input.insert("temperature".to_string(), Value::from(t));
        }
        if let Some(m) = self.config.max_tokens {
            input.insert("max_tokens".to_string(), Value::from(m));
        }
        let request = ReplicatePredictionRequest {
            input,
            stream: Some(true),
        };
        let stream = self.stream_request(request).await?;
        let mapped = stream.map(|chunk_result| match chunk_result {
            Ok(chunk) => Ok(MessageChunk::new(chunk.text)),
            Err(e) => Err(e),
        });
        Ok(Box::pin(mapped))
    }

    fn bind_functions(&self, functions: Vec<FunctionDefinition>) -> Arc<dyn ChatModel> {
        let mut new = self.clone();
        new.bound_functions = functions;
        Arc::new(new)
    }

    fn bind_tools(&self, tools: Vec<ToolDefinition>) -> Arc<dyn ChatModel> {
        let mut new = self.clone();
        new.bound_tools = tools;
        Arc::new(new)
    }
}
