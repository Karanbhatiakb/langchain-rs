//! HuggingFace inference LLM provider implementation.

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

const HF_INFERENCE_URL: &str = "https://api-inference.huggingface.co/models";

pub struct ChatHuggingFace {
    model: String,
    api_key: String,
    api_url: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatHuggingFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatHuggingFace")
            .field("model", &self.model)
            .field("api_url", &self.api_url)
            .finish()
    }
}

#[derive(Serialize)]
struct HFTextGenerationRequest {
    inputs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HFParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<HFOptions>,
}

#[derive(Serialize)]
struct HFParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_new_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    do_sample: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_full_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Serialize)]
struct HFOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_for_model: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_cache: Option<bool>,
}

#[derive(Deserialize)]
struct HFTextGenerationResponse {
    #[serde(default)]
    generated_text: String,
}

#[derive(Deserialize)]
struct HFErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct HFStreamToken {
    text: String,
}

#[derive(Deserialize)]
struct HFStreamResponse {
    token: HFStreamToken,
    generated_text: Option<String>,
}

impl ChatHuggingFace {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        let model_str = model.into();
        Self {
            api_url: format!("{}/{}", HF_INFERENCE_URL, model_str),
            model: model_str,
            api_key: api_key.into(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    async fn send_request(&self, request: HFTextGenerationRequest) -> Result<HFTextGenerationResponse> {
        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("HuggingFace request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "HuggingFace API error ({}): {}",
                status, body
            )));
        }

        Ok(response.json::<HFTextGenerationResponse>().await?)
    }

    async fn stream_request(
        &self,
        request: HFTextGenerationRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let client = self.client.clone();
        let api_url = self.api_url.clone();
        let api_key = self.api_key.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&api_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .header("Accept", "text/event-stream")
                .json(&request)
                .timeout(Duration::from_secs(120))
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("HF stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChainError::LLMError(format!(
                    "HF API streaming error ({}): {}",
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

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(stream_resp) = serde_json::from_str::<HFStreamResponse>(data) {
                            if stream_resp.generated_text.is_some() {
                                break;
                            }
                            if !stream_resp.token.text.is_empty() {
                                yield GenerationChunk::new(stream_resp.token.text);
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

impl Clone for ChatHuggingFace {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            api_url: self.api_url.clone(),
            config: self.config.clone(),
            client: Client::new(),
            callbacks: self.callbacks.clone(),
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for ChatHuggingFace {
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
        let prompt = prompts.join("\n");
        let request = HFTextGenerationRequest {
            inputs: prompt,
            parameters: Some(HFParameters {
                temperature: self.config.temperature,
                max_new_tokens: self.config.max_tokens,
                top_p: self.config.top_p,
                do_sample: self.config.temperature.map(|t| t > 0.0),
                return_full_text: Some(false),
                stop: stop
                    .map(|s| s.iter().map(|&s| s.to_string()).collect())
                    .or_else(|| self.config.stop_sequences.clone()),
            }),
            options: Some(HFOptions {
                wait_for_model: Some(true),
                use_cache: None,
            }),
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
impl ChatModel for ChatHuggingFace {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let prompt = messages
            .iter()
            .map(|m| {
                let role = match m.message_type {
                    MessageType::Human => "Human",
                    MessageType::AI => "Assistant",
                    MessageType::System => "System",
                    _ => "User",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let request = HFTextGenerationRequest {
            inputs: prompt,
            parameters: Some(HFParameters {
                temperature: self.config.temperature,
                max_new_tokens: self.config.max_tokens,
                top_p: self.config.top_p,
                do_sample: self.config.temperature.map(|t| t > 0.0),
                return_full_text: Some(false),
                stop: stop
                    .map(|s| s.iter().map(|&s| s.to_string()).collect())
                    .or_else(|| self.config.stop_sequences.clone()),
            }),
            options: Some(HFOptions {
                wait_for_model: Some(true),
                use_cache: None,
            }),
        };
        let response = self.send_request(request).await?;
        Ok(BaseMessage::new(response.generated_text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let prompt = messages
            .iter()
            .map(|m| {
                let role = match m.message_type {
                    MessageType::Human => "Human",
                    MessageType::AI => "Assistant",
                    MessageType::System => "System",
                    _ => "User",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let request = HFTextGenerationRequest {
            inputs: prompt,
            parameters: Some(HFParameters {
                temperature: self.config.temperature,
                max_new_tokens: self.config.max_tokens,
                top_p: self.config.top_p,
                do_sample: self.config.temperature.map(|t| t > 0.0),
                return_full_text: Some(false),
                stop: stop
                    .map(|s| s.iter().map(|&s| s.to_string()).collect())
                    .or_else(|| self.config.stop_sequences.clone()),
            }),
            options: Some(HFOptions {
                wait_for_model: Some(true),
                use_cache: None,
            }),
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
