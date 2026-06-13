//! SambaNova LLM provider implementation.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::{ChainError, Result};
use langchain_core::messages::{BaseMessage, MessageType};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

use crate::traits::{BaseLLM, ChatModel, FunctionDefinition, ToolDefinition};
use crate::types::{Generation, GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

const SAMBANOVA_BASE_URL: &str = "https://api.sambanova.ai/v1";

pub struct SambaNovaLLM {
    model: String,
    api_key: String,
    base_url: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    max_retries: u32,
    timeout: Duration,
}

impl std::fmt::Debug for SambaNovaLLM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SambaNovaLLM")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

pub struct ChatSambaNova {
    model: String,
    api_key: String,
    base_url: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    max_retries: u32,
    timeout: Duration,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatSambaNova {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatSambaNova")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct SambaNovaChatRequest {
    model: String,
    messages: Vec<SambaNovaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

#[derive(Serialize)]
struct SambaNovaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Deserialize)]
struct SambaNovaChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<SambaNovaChoice>,
    #[serde(default)]
    usage: Option<SambaNovaUsage>,
}

#[derive(Deserialize)]
struct SambaNovaChoice {
    index: u32,
    message: SambaNovaResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct SambaNovaResponseMessage {
    role: String,
    content: Option<String>,
}

#[derive(Deserialize)]
struct SambaNovaUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct SambaNovaStreamChunk {
    choices: Vec<SambaNovaStreamChoice>,
}

#[derive(Deserialize)]
struct SambaNovaStreamChoice {
    delta: SambaNovaDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct SambaNovaDelta {
    content: Option<String>,
    role: Option<String>,
}

impl SambaNovaLLM {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            base_url: SAMBANOVA_BASE_URL.to_string(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            max_retries: 3,
            timeout: Duration::from_secs(60),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn with_config(mut self, config: GenerationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_callbacks(mut self, callbacks: CallbackManager) -> Self {
        self.callbacks = callbacks;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn convert_messages(&self, prompts: &[String]) -> Vec<SambaNovaMessage> {
        prompts
            .iter()
            .map(|p| SambaNovaMessage {
                role: "user".to_string(),
                content: p.clone(),
                name: None,
            })
            .collect()
    }

    fn build_request(&self, messages: Vec<SambaNovaMessage>, stop: Option<&[&str]>, stream: bool) -> SambaNovaChatRequest {
        SambaNovaChatRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
            stream: if stream { Some(true) } else { None },
            n: self.config.n,
            user: self.config.user.clone(),
        }
    }

    async fn send_request(&self, request: SambaNovaChatRequest) -> Result<SambaNovaChatResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(self.timeout)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return Ok(resp.json::<SambaNovaChatResponse>().await.map_err(|e| {
                            ChainError::ParserError(e.to_string())
                        })?);
                    }

                    let status = resp.status();
                    let error_body = resp.text().await.unwrap_or_default();

                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        last_error = Some(ChainError::RateLimitError(
                            "Rate limited by SambaNova API".to_string(),
                        ));
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        warn!("Rate limited, retrying in {:?}", wait);
                        sleep(wait).await;
                        continue;
                    }

                    if status.is_server_error() {
                        last_error = Some(ChainError::LLMError(format!(
                            "SambaNova API error ({}): {}",
                            status, error_body
                        )));
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        sleep(wait).await;
                        continue;
                    }

                    return Err(ChainError::LLMError(format!(
                        "SambaNova API error ({}): {}",
                        status, error_body
                    )));
                }
                Err(e) => {
                    last_error = Some(ChainError::LLMError(format!(
                        "Request failed: {}",
                        e
                    )));
                    if attempt < self.max_retries - 1 {
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        sleep(wait).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ChainError::LLMError("Max retries exceeded".to_string())
        }))
    }

    async fn stream_request(
        &self,
        request: SambaNovaChatRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        let timeout = self.timeout;
        let client = self.client.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(timeout)
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let err = ChainError::LLMError(format!(
                    "SambaNova API streaming error ({})",
                    status
                ));
                Err(err)?;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| ChainError::LLMError(format!("Stream error: {}", e)))?;
                let chunk_str = String::from_utf8_lossy(&chunk);
                buffer.push_str(&chunk_str);

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if line == "data: [DONE]" {
                        break;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(chunk_data) = serde_json::from_str::<SambaNovaStreamChunk>(data) {
                            for choice in chunk_data.choices {
                                if let Some(content) = &choice.delta.content {
                                    yield GenerationChunk::new(content.clone());
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

impl Clone for SambaNovaLLM {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            config: self.config.clone(),
            client: Client::new(),
            callbacks: self.callbacks.clone(),
            max_retries: self.max_retries,
            timeout: self.timeout,
        }
    }
}

#[async_trait]
impl BaseLLM for SambaNovaLLM {
    async fn generate(&self, prompts: &[String], stop: Option<&[&str]>) -> Result<LLMResult> {
        let messages = self.convert_messages(prompts);
        let request = self.build_request(messages, stop, false);
        let response = self.send_request(request).await?;
        let text = response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();
        let generation = Generation {
            text,
            message: None,
            generation_info: None,
        };
        Ok(LLMResult {
            generations: vec![vec![generation]],
            llm_output: None,
        })
    }

    async fn stream(
        &self,
        prompts: &[String],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let messages = self.convert_messages(prompts);
        let request = self.build_request(messages, stop, true);
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

impl ChatSambaNova {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            base_url: SAMBANOVA_BASE_URL.to_string(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            max_retries: 3,
            timeout: Duration::from_secs(60),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn with_config(mut self, config: GenerationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_callbacks(mut self, callbacks: CallbackManager) -> Self {
        self.callbacks = callbacks;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn convert_messages(&self, messages: &[BaseMessage]) -> Vec<SambaNovaMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.message_type {
                    MessageType::Human => "user",
                    MessageType::AI => "assistant",
                    MessageType::System => "system",
                    MessageType::Tool => "tool",
                    MessageType::Function => "function",
                    MessageType::Generic | MessageType::Chat => "user",
                };
                SambaNovaMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                    name: msg.name.clone(),
                }
            })
            .collect()
    }

    fn build_request(&self, messages: Vec<SambaNovaMessage>, stop: Option<&[&str]>, stream: bool) -> SambaNovaChatRequest {
        SambaNovaChatRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
            stream: if stream { Some(true) } else { None },
            n: self.config.n,
            user: self.config.user.clone(),
        }
    }

    async fn send_request(&self, request: SambaNovaChatRequest) -> Result<SambaNovaChatResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(self.timeout)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return Ok(resp.json::<SambaNovaChatResponse>().await.map_err(|e| {
                            ChainError::ParserError(e.to_string())
                        })?);
                    }

                    let status = resp.status();
                    let error_body = resp.text().await.unwrap_or_default();

                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        last_error = Some(ChainError::RateLimitError(
                            "Rate limited by SambaNova API".to_string(),
                        ));
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        warn!("Rate limited, retrying in {:?}", wait);
                        sleep(wait).await;
                        continue;
                    }

                    if status.is_server_error() {
                        last_error = Some(ChainError::LLMError(format!(
                            "SambaNova API error ({}): {}",
                            status, error_body
                        )));
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        sleep(wait).await;
                        continue;
                    }

                    return Err(ChainError::LLMError(format!(
                        "SambaNova API error ({}): {}",
                        status, error_body
                    )));
                }
                Err(e) => {
                    last_error = Some(ChainError::LLMError(format!(
                        "Request failed: {}",
                        e
                    )));
                    if attempt < self.max_retries - 1 {
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        sleep(wait).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ChainError::LLMError("Max retries exceeded".to_string())
        }))
    }

    async fn stream_request(
        &self,
        request: SambaNovaChatRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        let timeout = self.timeout;
        let client = self.client.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(timeout)
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let err = ChainError::LLMError(format!(
                    "SambaNova API streaming error ({})",
                    status
                ));
                Err(err)?;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| ChainError::LLMError(format!("Stream error: {}", e)))?;
                let chunk_str = String::from_utf8_lossy(&chunk);
                buffer.push_str(&chunk_str);

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if line == "data: [DONE]" {
                        break;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(chunk_data) = serde_json::from_str::<SambaNovaStreamChunk>(data) {
                            for choice in chunk_data.choices {
                                if let Some(content) = &choice.delta.content {
                                    yield GenerationChunk::new(content.clone());
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

impl Clone for ChatSambaNova {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            config: self.config.clone(),
            client: Client::new(),
            callbacks: self.callbacks.clone(),
            max_retries: self.max_retries,
            timeout: self.timeout,
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for ChatSambaNova {
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
        let converted = self.convert_messages(&messages);
        let request = self.build_request(converted, stop, true);
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
impl ChatModel for ChatSambaNova {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let converted = self.convert_messages(messages);
        let request = self.build_request(converted, stop, false);
        let response = self.send_request(request).await?;
        let text = response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();
        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let converted = self.convert_messages(messages);
        let request = self.build_request(converted, stop, true);
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
