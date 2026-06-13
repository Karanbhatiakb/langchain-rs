//! Anthropic Claude LLM provider implementation.

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
use tracing::warn;

use crate::traits::{BaseLLM, ChatModel, FunctionDefinition, ToolDefinition};
use crate::types::{Generation, GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
const DEFAULT_MODEL: &str = "claude-3-5-sonnet-20241022";

pub struct ChatAnthropic {
    model: String,
    api_key: String,
    base_url: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    max_tokens: u32,
    max_retries: u32,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatAnthropic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatAnthropic")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<ContentBlock>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ContentBlock {
    Text { text: String, #[serde(rename = "type")] content_type: String },
    ToolUse { id: String, name: String, input: Value, #[serde(rename = "type")] content_type: String },
    ToolResult { tool_use_id: String, content: String, #[serde(rename = "type")] content_type: String },
}

#[derive(Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Deserialize)]
struct MessagesResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ResponseContent>,
    model: String,
    stop_reason: Option<String>,
    usage: Option<UsageInfo>,
}

#[derive(Deserialize)]
struct ResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    input: Option<Value>,
}

#[derive(Deserialize)]
struct UsageInfo {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<StreamDelta>,
    #[serde(default)]
    content_block: Option<StreamContentBlock>,
}

#[derive(Deserialize)]
struct StreamDelta {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct StreamContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: Option<String>,
}

impl ChatAnthropic {
    pub fn new(model: Option<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            api_key: api_key.into(),
            base_url: ANTHROPIC_BASE_URL.to_string(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            max_tokens: 1024,
            max_retries: 3,
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

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    fn convert_messages(&self, messages: &[BaseMessage]) -> (Vec<AnthropicMessage>, Option<String>) {
        let mut system = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.message_type {
                MessageType::System => {
                    system = Some(msg.content.clone());
                }
                MessageType::Human => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: vec![ContentBlock::Text {
                            text: msg.content.clone(),
                            content_type: "text".to_string(),
                        }],
                    });
                }
                MessageType::AI => {
                    let mut content = vec![ContentBlock::Text {
                        text: msg.content.clone(),
                        content_type: "text".to_string(),
                    }];
                    if let Some(tool_calls) = msg.additional_kwargs.get("tool_calls") {
                        if let Some(calls) = tool_calls.as_array() {
                            for call in calls {
                                if let (Some(id), Some(name), Some(input)) = (
                                    call.get("id").and_then(|v| v.as_str()),
                                    call.get("function").and_then(|f| f.get("name").and_then(|v| v.as_str())),
                                    call.get("function").and_then(|f| f.get("arguments")),
                                ) {
                                    content.push(ContentBlock::ToolUse {
                                        id: id.to_string(),
                                        name: name.to_string(),
                                        input: input.clone(),
                                        content_type: "tool_use".to_string(),
                                    });
                                }
                            }
                        }
                    }
                    anthropic_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content,
                    });
                }
                MessageType::Tool => {
                    let tool_use_id = msg
                        .additional_kwargs
                        .get("tool_call_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: vec![ContentBlock::ToolResult {
                            tool_use_id,
                            content: msg.content.clone(),
                            content_type: "tool_result".to_string(),
                        }],
                    });
                }
                MessageType::Function => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: vec![ContentBlock::Text {
                            text: msg.content.clone(),
                            content_type: "text".to_string(),
                        }],
                    });
                }
                MessageType::Generic => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: vec![ContentBlock::Text {
                            text: msg.content.clone(),
                            content_type: "text".to_string(),
                        }],
                    });
                }
            }
        }

        (anthropic_messages, system)
    }

    fn build_tools(&self, functions: Option<&[FunctionDefinition]>) -> Option<Vec<AnthropicTool>> {
        let mut all_tools = self.bound_tools.clone();
        if let Some(funcs) = functions {
            all_tools.extend(
                funcs
                    .iter()
                    .map(|f| ToolDefinition {
                        name: f.name.clone(),
                        description: f.description.clone(),
                        parameters: f.parameters.clone(),
                    }),
            );
        }
        let all_tools: Vec<FunctionDefinition> = self
            .bound_functions
            .iter()
            .chain(all_tools.iter())
            .cloned()
            .collect();

        if all_tools.is_empty() {
            return None;
        }

        Some(
            all_tools
                .into_iter()
                .map(|t| AnthropicTool {
                    name: t.name,
                    description: t.description,
                    input_schema: t.parameters,
                })
                .collect(),
        )
    }

    async fn send_request(&self, request: MessagesRequest) -> Result<MessagesResponse> {
        let url = format!("{}/messages", self.base_url);
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            let response = self
                .client
                .post(&url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(Duration::from_secs(60))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return Ok(resp.json::<MessagesResponse>().await?);
                    }
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        last_error = Some(ChainError::LLMError(format!(
                            "Anthropic API error ({}): {}",
                            status, body
                        )));
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        warn!("Retrying in {:?}", wait);
                        tokio::time::sleep(wait).await;
                        continue;
                    }
                    return Err(ChainError::LLMError(format!(
                        "Anthropic API error ({}): {}",
                        status, body
                    )));
                }
                Err(e) => {
                    last_error = Some(ChainError::LLMError(format!("Request failed: {}", e)));
                    if attempt < self.max_retries - 1 {
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        tokio::time::sleep(wait).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ChainError::LLMError("Max retries exceeded".to_string())))
    }

    async fn stream_request(
        &self,
        request: MessagesRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!("{}/messages", self.base_url);
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChainError::LLMError(format!(
                    "Anthropic API streaming error ({}): {}",
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

                    if line.is_empty() || !line.starts_with("data: ") {
                        continue;
                    }

                    let data = &line[6..];
                    if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                        if let Some(delta) = event.delta {
                            if let Some(text) = delta.text {
                                yield GenerationChunk::new(text);
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    fn convert_response_to_message(&self, response: MessagesResponse) -> BaseMessage {
        let mut content = String::new();
        let mut additional_kwargs = HashMap::new();
        let mut tool_calls = Vec::new();

        for block in &response.content {
            match block.content_type.as_str() {
                "text" => {
                    if let Some(ref text) = block.text {
                        content.push_str(text);
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name), Some(input)) = (&block.id, &block.name, &block.input) {
                        let tool_call = serde_json::json!({
                            "id": id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": input.to_string(),
                            }
                        });
                        tool_calls.push(tool_call);
                    }
                }
                _ => {}
            }
        }

        if !tool_calls.is_empty() {
            additional_kwargs.insert("tool_calls".to_string(), Value::Array(tool_calls));
        }
        if let Some(reason) = &response.stop_reason {
            additional_kwargs
                .insert("finish_reason".to_string(), Value::String(reason.clone()));
        }

        BaseMessage::new(content, MessageType::AI).with_additional_kwargs(additional_kwargs)
    }
}

impl Clone for ChatAnthropic {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            config: self.config.clone(),
            client: Client::new(),
            callbacks: self.callbacks.clone(),
            max_tokens: self.max_tokens,
            max_retries: self.max_retries,
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for ChatAnthropic {
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
        let (anthropic_messages, system) = self.convert_messages(&messages);
        let request = MessagesRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            max_tokens: self.max_tokens,
            messages: anthropic_messages,
            system,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            stop_sequences: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
            tools: None,
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
impl ChatModel for ChatAnthropic {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let (anthropic_messages, system) = self.convert_messages(messages);
        let tools = self.build_tools(functions);
        let request = MessagesRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            max_tokens: self.max_tokens,
            messages: anthropic_messages,
            system,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            stop_sequences: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
            tools,
            stream: None,
        };
        let response = self.send_request(request).await?;
        Ok(self.convert_response_to_message(response))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let (anthropic_messages, system) = self.convert_messages(messages);
        let request = MessagesRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            max_tokens: self.max_tokens,
            messages: anthropic_messages,
            system,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            stop_sequences: stop
                .map(|s| s.iter().map(|&s| s.to_string()).collect())
                .or_else(|| self.config.stop_sequences.clone()),
            tools: None,
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
