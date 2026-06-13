//! OpenAI / Azure OpenAI LLM provider implementation.

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::{ChainError, Result};
use langchain_core::messages::{BaseMessage, MessageType};
use reqwest::header::{HeaderMap, HeaderValue, HeaderName};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

use crate::traits::{BaseLLM, ChatModel, FunctionDefinition, ToolDefinition};
use crate::types::{Generation, GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

pub struct ChatOpenAI {
    model: String,
    api_key: String,
    base_url: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    max_retries: u32,
    timeout: Duration,
    organization: Option<String>,
    default_headers: HashMap<String, String>,
    streaming: bool,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatOpenAI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatOpenAI")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
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
    functions: Option<Vec<FunctionDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolDefinition,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Choice {
    index: u32,
    message: ResponseMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ResponseMessage {
    role: String,
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<Value>>,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct StreamChoice {
    delta: Delta,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<Value>>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetail,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ApiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

impl ChatOpenAI {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            base_url: OPENAI_BASE_URL.to_string(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            max_retries: 3,
            timeout: Duration::from_secs(60),
            organization: None,
            default_headers: HashMap::new(),
            streaming: false,
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

    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    pub fn with_default_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.default_headers = headers;
        self
    }

    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    fn convert_messages(&self, messages: &[BaseMessage]) -> Vec<Message> {
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
                }
                .to_string();

                let function_call = msg.additional_kwargs.get("function_call").cloned();
                let tool_calls = msg.additional_kwargs.get("tool_calls").and_then(|v| {
                    v.as_array().map(|arr| arr.clone())
                });
                let tool_call_id = msg
                    .additional_kwargs
                    .get("tool_call_id")
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                Message {
                    role,
                    content: msg.content.clone(),
                    name: msg.name.clone(),
                    function_call,
                    tool_calls,
                    tool_call_id,
                }
            })
            .collect()
    }

    fn build_request(
        &self,
        messages: Vec<Message>,
        functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
        stream: bool,
    ) -> ChatCompletionRequest {
        let mut all_functions = self.bound_functions.clone();
        if let Some(funcs) = functions {
            all_functions.extend_from_slice(funcs);
        }
        let all_functions = if all_functions.is_empty() {
            None
        } else {
            Some(all_functions)
        };

        let all_tools = if !self.bound_tools.is_empty() {
            Some(
                self.bound_tools
                    .iter()
                    .map(|t| Tool {
                        tool_type: "function".to_string(),
                        function: t.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        let stop_vec = stop
            .map(|s| s.iter().map(|&s| s.to_string()).collect())
            .or_else(|| self.config.stop_sequences.clone());

        let has_functions = all_functions.is_some();
        let has_tools = all_tools.is_some();

        ChatCompletionRequest {
            model: self.config.model.clone().unwrap_or_else(|| self.model.clone()),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: stop_vec,
            functions: all_functions,
            function_call: if has_functions && stream {
                Some(Value::String("auto".to_string()))
            } else {
                None
            },
            tools: all_tools,
            tool_choice: if has_tools {
                Some(Value::String("auto".to_string()))
            } else {
                None
            },
            seed: self.config.seed,
            n: self.config.n,
            user: self.config.user.clone(),
            stream: if stream { Some(true) } else { None },
        }
    }

    fn convert_response_to_message(&self, response: ChatCompletionResponse) -> BaseMessage {
        let choice = response.choices.into_iter().next().unwrap();
        let content = choice.message.content.unwrap_or_default();
        let mut additional_kwargs = HashMap::new();

        if let Some(fc) = choice.message.function_call {
            additional_kwargs.insert("function_call".to_string(), fc);
        }
        if let Some(tc) = choice.message.tool_calls {
            additional_kwargs.insert("tool_calls".to_string(), Value::Array(tc));
        }
        if let Some(reason) = choice.finish_reason {
            additional_kwargs
                .insert("finish_reason".to_string(), Value::String(reason));
        }

        let msg_type = match choice.message.role.as_str() {
            "assistant" => MessageType::AI,
            _ => MessageType::AI,
        };

        BaseMessage::new(content, msg_type).with_additional_kwargs(additional_kwargs)
    }

    #[allow(dead_code)]
    fn convert_message_to_base(&self, msg: &BaseMessage) -> BaseMessage {
        msg.clone()
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        if let Some(ref org) = self.organization {
            headers.insert(
                HeaderName::from_static("openai-organization"),
                HeaderValue::from_str(org).unwrap(),
            );
        }
        for (key, value) in &self.default_headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                headers.insert(name, val);
            }
        }
        headers
    }

    async fn send_request(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let headers = self.build_headers();

        let mut last_error = None;
        for attempt in 0..self.max_retries {
            let response = self
                .client
                .post(&url)
                .headers(headers.clone())
                .json(&request)
                .timeout(self.timeout)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return Ok(resp.json::<ChatCompletionResponse>().await.map_err(|e| ChainError::ParserError(e.to_string()))?);
                    }

                    let status = resp.status();
                    let error_body = resp.text().await.unwrap_or_default();

                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        last_error = Some(ChainError::RateLimitError("Rate limited by OpenAI API".to_string()));
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        warn!("Rate limited, retrying in {:?}", wait);
                        sleep(wait).await;
                        continue;
                    }

                    if status.is_server_error() {
                        last_error = Some(ChainError::LLMError(format!(
                            "OpenAI API error ({}): {}",
                            status, error_body
                        )));
                        let wait = Duration::from_millis(2u64.pow(attempt) * 1000);
                        sleep(wait).await;
                        continue;
                    }

                    return Err(ChainError::LLMError(format!(
                        "OpenAI API error ({}): {}",
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
        request: ChatCompletionRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!("{}/chat/completions", self.base_url);
        let headers = self.build_headers();
        let client = self.client.clone();
        let timeout = self.timeout;

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .headers(headers)
                .json(&request)
                .timeout(timeout)
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let err = ChainError::LLMError(format!(
                    "OpenAI API streaming error ({})",
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
                        if let Ok(chunk_data) = serde_json::from_str::<StreamChunk>(data) {
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

pub fn num_tokens_from_messages(messages: &[BaseMessage], model: &str) -> u32 {
    let tokens_per_message = if model.contains("gpt-4") || model.contains("gpt-3.5-turbo") {
        3
    } else {
        3
    };
    let tokens_per_name = if model.contains("gpt-4") || model.contains("gpt-3.5-turbo") {
        1
    } else {
        1
    };

    let mut total: u32 = messages.iter().map(|msg| {
        let text_len = msg.content.len() as u32;
        let content_tokens = (text_len + 3) / 4;
        let name_tokens = msg.name.as_ref().map(|_| tokens_per_name).unwrap_or(0);
        tokens_per_message + content_tokens + name_tokens
    }).sum();

    total += 3;
    total
}

pub fn get_model_context_length(model: &str) -> u32 {
    match model {
        m if m.contains("gpt-4o") => 128_000,
        m if m.contains("gpt-4-turbo") => 128_000,
        m if m.contains("gpt-4-32k") => 32_768,
        m if m.contains("gpt-4") => 8_192,
        m if m.contains("gpt-3.5-turbo-16k") => 16_384,
        m if m.contains("gpt-3.5-turbo") => 16_384,
        _ => 4_096,
    }
}

#[async_trait]
impl BaseLLM for ChatOpenAI {
    async fn generate(
        &self,
        prompts: &[String],
        stop: Option<&[&str]>,
    ) -> Result<LLMResult> {
        let messages: Vec<BaseMessage> = prompts
            .iter()
            .map(|p| BaseMessage::new(p.clone(), MessageType::Human))
            .collect();

        let msg = self.predict_messages(&messages, None, stop).await?;
        let generation = Generation {
            text: msg.content.clone(),
            message: Some(msg),
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
        let messages: Vec<BaseMessage> = prompts
            .iter()
            .map(|p| BaseMessage::new(p.clone(), MessageType::Human))
            .collect();

        let messages = self.convert_messages(&messages);
        let request = self.build_request(messages, None, stop, true);
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

impl Clone for ChatOpenAI {
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
            organization: self.organization.clone(),
            default_headers: self.default_headers.clone(),
            streaming: self.streaming,
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl ChatModel for ChatOpenAI {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let converted = self.convert_messages(messages);
        let request = self.build_request(converted, functions, stop, false);
        let response = self.send_request(request).await?;
        Ok(self.convert_response_to_message(response))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let messages = self.convert_messages(messages);
        let request = self.build_request(messages, None, stop, true);

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
