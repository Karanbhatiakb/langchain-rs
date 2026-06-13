//! Ollama local LLM provider implementation.

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

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

pub struct ChatOllama {
    model: String,
    base_url: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatOllama {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatOllama")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    model: String,
    created_at: String,
    message: OllamaResponseMessage,
    done: bool,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaStreamChunk {
    message: Option<OllamaResponseMessage>,
    done: bool,
}

impl ChatOllama {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: OLLAMA_BASE_URL.to_string(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    fn convert_messages(&self, messages: &[BaseMessage]) -> Vec<OllamaMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.message_type {
                    MessageType::Human => "user",
                    MessageType::AI => "assistant",
                    MessageType::System => "system",
                    _ => "user",
                };
                OllamaMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                }
            })
            .collect()
    }

    async fn send_request(&self, request: OllamaChatRequest) -> Result<OllamaChatResponse> {
        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Ollama API error ({}): {}",
                status, body
            )));
        }

        Ok(response.json::<OllamaChatResponse>().await?)
    }

    async fn stream_request(
        &self,
        request: OllamaChatRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!("{}/api/chat", self.base_url);
        let client = self.client.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Ollama stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChainError::LLMError(format!(
                    "Ollama API streaming error ({}): {}",
                    status, body
                )))?;
            }

            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk.map_err(|e| ChainError::LLMError(format!("Stream error: {}", e)))?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if let Ok(chunk_data) = serde_json::from_str::<OllamaStreamChunk>(&line) {
                        if chunk_data.done {
                            break;
                        }
                        if let Some(msg) = chunk_data.message {
                            if !msg.content.is_empty() {
                                yield GenerationChunk::new(msg.content);
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

impl Clone for ChatOllama {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            base_url: self.base_url.clone(),
            config: self.config.clone(),
            client: Client::new(),
            callbacks: self.callbacks.clone(),
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for ChatOllama {
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
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let messages: Vec<BaseMessage> = prompts
            .iter()
            .map(|p| BaseMessage::new(p.clone(), MessageType::Human))
            .collect();
        let ollama_messages = self.convert_messages(&messages);
        let options = OllamaOptions {
            temperature: self.config.temperature,
            num_predict: self.config.max_tokens,
            top_p: self.config.top_p,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
        };
        let request = OllamaChatRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages: ollama_messages,
            options: Some(options),
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
impl ChatModel for ChatOllama {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let ollama_messages = self.convert_messages(messages);
        let options = OllamaOptions {
            temperature: self.config.temperature,
            num_predict: self.config.max_tokens,
            top_p: self.config.top_p,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
        };
        let request = OllamaChatRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages: ollama_messages,
            options: Some(options),
            stream: None,
        };
        let response = self.send_request(request).await?;
        Ok(BaseMessage::new(response.message.content, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let ollama_messages = self.convert_messages(messages);
        let options = OllamaOptions {
            temperature: self.config.temperature,
            num_predict: self.config.max_tokens,
            top_p: self.config.top_p,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
        };
        let request = OllamaChatRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages: ollama_messages,
            options: Some(options),
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
