//! Google Gemini LLM provider implementation.

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

const DEFAULT_MODEL: &str = "gemini-pro";

pub struct ChatGoogle {
    model: String,
    api_key: String,
    base_url: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatGoogle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatGoogle")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfigValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_settings: Option<Vec<SafetySetting>>,
}

#[derive(Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: Blob },
    FunctionCall { function_call: FunctionCallValue },
    FunctionResponse { function_response: FunctionResponseValue },
}

#[derive(Serialize)]
struct Blob {
    mime_type: String,
    data: String,
}

#[derive(Serialize)]
struct FunctionCallValue {
    name: String,
    args: Value,
}

#[derive(Serialize)]
struct FunctionResponseValue {
    name: String,
    response: Value,
}

#[derive(Serialize)]
struct GenerationConfigValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Serialize)]
struct SafetySetting {
    category: String,
    threshold: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    #[serde(default)]
    prompt_feedback: Option<PromptFeedback>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<ContentResponse>,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    safety_ratings: Vec<SafetyRating>,
}

#[derive(Deserialize)]
struct ContentResponse {
    role: String,
    parts: Vec<PartResponse>,
}

#[derive(Deserialize)]
struct PartResponse {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    function_call: Option<FunctionCallResponse>,
}

#[derive(Deserialize)]
struct FunctionCallResponse {
    name: String,
    args: Value,
}

#[derive(Deserialize)]
struct SafetyRating {
    category: String,
    probability: String,
}

#[derive(Deserialize)]
struct PromptFeedback {
    safety_ratings: Vec<SafetyRating>,
}

impl ChatGoogle {
    pub fn new(model: Option<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            api_key: api_key.into(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
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

    pub fn with_config(mut self, config: GenerationConfig) -> Self {
        self.config = config;
        self
    }

    fn convert_messages(&self, messages: &[BaseMessage]) -> Vec<Content> {
        let mut contents = Vec::new();
        for msg in messages {
            let role = match msg.message_type {
                MessageType::Human => "user",
                MessageType::AI => "model",
                MessageType::System => "user",
                _ => "user",
            };
            contents.push(Content {
                role: role.to_string(),
                parts: vec![Part::Text {
                    text: msg.content.clone(),
                }],
            });
        }
        contents
    }

    async fn send_request(&self, request: GeminiRequest) -> Result<GeminiResponse> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ChainError::LLMError(format!("Google API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Google API error ({}): {}",
                status, body
            )));
        }

        Ok(response.json::<GeminiResponse>().await?)
    }

    fn convert_response_to_message(&self, response: GeminiResponse) -> BaseMessage {
        let mut content = String::new();
        let mut additional_kwargs = HashMap::new();

        if let Some(candidate) = response.candidates.into_iter().next() {
            if let Some(content_resp) = candidate.content {
                for part in content_resp.parts {
                    if let Some(text) = part.text {
                        content.push_str(&text);
                    }
                    if let Some(fc) = part.function_call {
                        let tool_call = serde_json::json!({
                            "function_call": {
                                "name": fc.name,
                                "arguments": fc.args,
                            }
                        });
                        additional_kwargs
                            .insert("function_call".to_string(), tool_call);
                    }
                }
            }
            if let Some(reason) = candidate.finish_reason {
                additional_kwargs
                    .insert("finish_reason".to_string(), Value::String(reason));
            }
        }

        BaseMessage::new(content, MessageType::AI).with_additional_kwargs(additional_kwargs)
    }

    async fn stream_request(
        &self,
        request: GeminiRequest,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.base_url, self.model, self.api_key
        );
        let client = self.client.clone();

        let stream = async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(Duration::from_secs(60))
                .send()
                .await
                .map_err(|e| ChainError::LLMError(format!("Google stream request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChainError::LLMError(format!(
                    "Google API streaming error ({}): {}",
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

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            break;
                        }

                        let candidates: Vec<Candidate> =
                            if let Ok(resp) = serde_json::from_str::<GeminiResponse>(data) {
                                resp.candidates
                            } else if let Ok(arr) = serde_json::from_str::<Vec<GeminiResponse>>(data) {
                                arr.into_iter().flat_map(|r| r.candidates).collect()
                            } else {
                                Vec::new()
                            };

                        for candidate in candidates {
                            if let Some(content) = candidate.content {
                                for part in content.parts {
                                    if let Some(text) = part.text {
                                        yield GenerationChunk::new(text);
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
}

impl Clone for ChatGoogle {
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
impl BaseLLM for ChatGoogle {
    async fn generate(&self, prompts: &[String], _stop: Option<&[&str]>) -> Result<LLMResult> {
        let messages: Vec<BaseMessage> = prompts
            .iter()
            .map(|p| BaseMessage::new(p.clone(), MessageType::Human))
            .collect();
        let msg = self.predict_messages(&messages, None, None).await?;
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
        let messages: Vec<BaseMessage> = prompts
            .iter()
            .map(|p| BaseMessage::new(p.clone(), MessageType::Human))
            .collect();
        let contents = self.convert_messages(&messages);
        let gen_config = GenerationConfigValue {
            temperature: self.config.temperature,
            max_output_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            stop_sequences: None,
        };
        let request = GeminiRequest {
            contents,
            generation_config: Some(gen_config),
            safety_settings: None,
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
impl ChatModel for ChatGoogle {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        _stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let contents = self.convert_messages(messages);
        let gen_config = GenerationConfigValue {
            temperature: self.config.temperature,
            max_output_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            stop_sequences: None,
        };
        let request = GeminiRequest {
            contents,
            generation_config: Some(gen_config),
            safety_settings: None,
        };
        let response = self.send_request(request).await?;
        Ok(self.convert_response_to_message(response))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        _stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let contents = self.convert_messages(messages);
        let gen_config = GenerationConfigValue {
            temperature: self.config.temperature,
            max_output_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            stop_sequences: None,
        };
        let request = GeminiRequest {
            contents,
            generation_config: Some(gen_config),
            safety_settings: None,
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
