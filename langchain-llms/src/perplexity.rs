//! Perplexity AI LLM provider implementation.

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

const PERPLEXITY_BASE_URL: &str = "https://api.perplexity.ai";

pub struct ChatPerplexity {
    model: String,
    api_key: String,
    base_url: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatPerplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatPerplexity")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct PerplexityChatRequest {
    model: String,
    messages: Vec<PerplexityMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize)]
struct PerplexityMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct PerplexityChatResponse {
    id: String,
    model: String,
    choices: Vec<PerplexityChoice>,
    usage: Option<PerplexityUsage>,
}

#[derive(Deserialize)]
struct PerplexityChoice {
    index: u32,
    message: PerplexityResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct PerplexityResponseMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct PerplexityUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct PerplexityStreamChunk {
    choices: Vec<PerplexityStreamChoice>,
}

#[derive(Deserialize)]
struct PerplexityStreamChoice {
    delta: PerplexityDelta,
}

#[derive(Deserialize)]
struct PerplexityDelta {
    #[serde(default)]
    content: Option<String>,
}

impl ChatPerplexity {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            base_url: PERPLEXITY_BASE_URL.to_string(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    fn convert_messages(&self, messages: &[BaseMessage]) -> Vec<PerplexityMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.message_type {
                    MessageType::Human => "user",
                    MessageType::AI => "assistant",
                    MessageType::System => "system",
                    _ => "user",
                };
                PerplexityMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                }
            })
            .collect()
    }

    async fn send_request(&self, request: PerplexityChatRequest) -> Result<PerplexityChatResponse> {
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Perplexity request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Perplexity API error ({}): {}",
                status, body
            )));
        }

        Ok(response.json::<PerplexityChatResponse>().await?)
    }

    async fn stream_request(
        &self,
        request: PerplexityChatRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Perplexity stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChainError::LLMError(format!(
                    "Perplexity API streaming error ({}): {}",
                    status, body
                )))?;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| ChainError::LLMError(format!("Stream error: {}", e)))?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

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
                        if let Ok(chunk_data) = serde_json::from_str::<PerplexityStreamChunk>(data) {
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

impl Clone for ChatPerplexity {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            api_key: self.api_key.clone(),
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
impl BaseLLM for ChatPerplexity {
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
        let perp_messages = self.convert_messages(&messages);
        let request = PerplexityChatRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages: perp_messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
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
impl ChatModel for ChatPerplexity {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let perp_messages = self.convert_messages(messages);
        let request = PerplexityChatRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages: perp_messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
            stream: None,
        };
        let response = self.send_request(request).await?;
        let text = response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();
        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let perp_messages = self.convert_messages(messages);
        let request = PerplexityChatRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages: perp_messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
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
