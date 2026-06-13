//! Azure OpenAI LLM provider implementation.

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

pub struct ChatAzure {
    deployment_name: String,
    api_key: String,
    base_url: String,
    api_version: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatAzure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatAzure")
            .field("deployment_name", &self.deployment_name)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct AzureChatRequest {
    messages: Vec<AzureMessage>,
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
struct AzureMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AzureChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<AzureChoice>,
    usage: Option<AzureUsage>,
}

#[derive(Deserialize)]
struct AzureChoice {
    index: u32,
    message: AzureResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct AzureResponseMessage {
    role: String,
    content: Option<String>,
}

#[derive(Deserialize)]
struct AzureUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct AzureStreamChunk {
    choices: Vec<AzureStreamChoice>,
}

#[derive(Deserialize)]
struct AzureStreamChoice {
    delta: AzureDelta,
}

#[derive(Deserialize)]
struct AzureDelta {
    #[serde(default)]
    content: Option<String>,
}

impl ChatAzure {
    pub fn new(
        deployment_name: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            deployment_name: deployment_name.into(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            api_version: "2024-02-15-preview".to_string(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    fn convert_messages(&self, messages: &[BaseMessage]) -> Vec<AzureMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.message_type {
                    MessageType::Human => "user",
                    MessageType::AI => "assistant",
                    MessageType::System => "system",
                    _ => "user",
                };
                AzureMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                }
            })
            .collect()
    }

    async fn send_request(&self, request: AzureChatRequest) -> Result<AzureChatResponse> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.base_url, self.deployment_name, self.api_version
        );

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Azure request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Azure API error ({}): {}",
                status, body
            )));
        }

        Ok(response.json::<AzureChatResponse>().await?)
    }

    async fn stream_request(
        &self,
        request: AzureChatRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.base_url, self.deployment_name, self.api_version
        );
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("api-key", &api_key)
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Azure stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChainError::LLMError(format!(
                    "Azure API streaming error ({}): {}",
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
                        if let Ok(chunk_data) = serde_json::from_str::<AzureStreamChunk>(data) {
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

impl Clone for ChatAzure {
    fn clone(&self) -> Self {
        Self {
            deployment_name: self.deployment_name.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            api_version: self.api_version.clone(),
            config: self.config.clone(),
            client: Client::new(),
            callbacks: self.callbacks.clone(),
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for ChatAzure {
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
        let azure_messages = self.convert_messages(&messages);
        let request = AzureChatRequest {
            messages: azure_messages,
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
impl ChatModel for ChatAzure {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let azure_messages = self.convert_messages(messages);
        let request = AzureChatRequest {
            messages: azure_messages,
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
            .and_then(|c| c.message.content)
            .unwrap_or_default();
        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let azure_messages = self.convert_messages(messages);
        let request = AzureChatRequest {
            messages: azure_messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stop: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
            stream: Some(true),
        };
        let stream = self.stream_request(request).await?;
        Ok(Box::pin(stream.map(|r| r.map(|c| MessageChunk::new(c.text)))))
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
