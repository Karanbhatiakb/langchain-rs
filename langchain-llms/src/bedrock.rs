//! AWS Bedrock LLM provider implementation.

use async_trait::async_trait;
use async_stream::try_stream;
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{sign, SigningSettings, SigningParams, SignableRequest};
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::{ChainError, Result};
use langchain_core::messages::{BaseMessage, MessageType};
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use url::Url;

use crate::traits::{BaseLLM, ChatModel, FunctionDefinition, ToolDefinition};
use crate::types::{Generation, GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

pub struct ChatBedrock {
    model: String,
    config: GenerationConfig,
    client: Client,
    callbacks: CallbackManager,
    region: String,
    access_key: String,
    secret_key: String,
    session_token: Option<String>,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for ChatBedrock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatBedrock")
            .field("model", &self.model)
            .field("region", &self.region)
            .finish()
    }
}

impl ChatBedrock {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            config: GenerationConfig::default(),
            client: Client::new(),
            callbacks: CallbackManager::new(),
            region: std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            access_key: std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            secret_key: std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    pub fn with_credentials(mut self, access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        self.access_key = access_key.into();
        self.secret_key = secret_key.into();
        self
    }

    pub fn with_session_token(mut self, token: impl Into<String>) -> Self {
        self.session_token = Some(token.into());
        self
    }

    fn credentials(&self) -> Credentials {
        Credentials::new(
            self.access_key.clone(),
            self.secret_key.clone(),
            self.session_token.clone(),
            None,
            "bedrock",
        )
    }
}

impl Clone for ChatBedrock {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            config: self.config.clone(),
            client: Client::new(),
            callbacks: self.callbacks.clone(),
            region: self.region.clone(),
            access_key: self.access_key.clone(),
            secret_key: self.secret_key.clone(),
            session_token: self.session_token.clone(),
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

async fn invoke_bedrock(
    client: &Client,
    method: &str,
    url: &str,
    region: &str,
    credentials: &Credentials,
    body: &[u8],
) -> Result<reqwest::Response> {
    let parsed_url = Url::parse(url).map_err(|e| ChainError::LLMError(format!("Invalid URL: {}", e)))?;
    let host = parsed_url.host_str().unwrap_or("").to_string();

    let http_req = http::Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", "application/json")
        .header("Host", &host)
        .body(Vec::from(body))
        .map_err(|e| ChainError::LLMError(format!("Failed to build request: {}", e)))?;

    let signable = SignableRequest::from(&http_req);

    let signing_params = SigningParams::builder()
        .access_key(credentials.access_key_id())
        .secret_key(credentials.secret_access_key())
        .security_token(credentials.session_token().map(|s| s.to_string()))
        .region(region)
        .service_name("bedrock")
        .time(SystemTime::now())
        .settings(SigningSettings::default())
        .build()
        .map_err(|e| ChainError::LLMError(format!("Failed to build signing params: {}", e)))?;

    let (signed_req, _) = sign(signable, &signing_params.into())
        .map_err(|e| ChainError::LLMError(format!("Failed to sign request: {}", e)))?;

    let signed_headers = signed_req.headers();
    let mut reqwest_headers = reqwest::header::HeaderMap::new();
    for (name, value) in signed_headers {
        if let Ok(h_name) = reqwest::header::HeaderName::from_bytes(name.as_ref()) {
            if let Ok(h_value) = reqwest::header::HeaderValue::from_bytes(value.as_ref()) {
                reqwest_headers.insert(h_name, h_value);
            }
        }
    }

    let http_method = match method {
        "POST" => reqwest::Method::POST,
        "GET" => reqwest::Method::GET,
        _ => reqwest::Method::POST,
    };

    let response = client
        .request(http_method, url)
        .headers(reqwest_headers)
        .body(body.to_vec())
        .send()
        .await
        .map_err(|e| ChainError::LLMError(format!("Bedrock request failed: {}", e)))?;

    Ok(response)
}

fn convert_messages_to_converse(messages: &[BaseMessage]) -> (Value, Value) {
    let mut system = Vec::new();
    let mut converse_messages = Vec::new();

    for msg in messages {
        match msg.message_type {
            MessageType::System => {
                system.push(json!({"text": msg.content}));
            }
            MessageType::AI => {
                converse_messages.push(json!({
                    "role": "assistant",
                    "content": [{"text": msg.content}]
                }));
            }
            _ => {
                converse_messages.push(json!({
                    "role": "user",
                    "content": [{"text": msg.content}]
                }));
            }
        }
    }

    let system_val = if system.is_empty() {
        Value::Null
    } else {
        json!(system)
    };

    (system_val, json!(converse_messages))
}

fn extract_event_payloads(data: &[u8]) -> (Vec<Vec<u8>>, usize) {
    let mut payloads = Vec::new();
    let mut pos = 0;
    while pos + 12 <= data.len() {
        let total_len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        if pos + total_len > data.len() {
            break;
        }
        let headers_len =
            u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]) as usize;
        let payload_start = pos + 12 + headers_len;
        let payload_end = pos + total_len - 4;
        if payload_end > payload_start {
            payloads.push(data[payload_start..payload_end].to_vec());
        }
        pos += total_len;
    }
    (payloads, pos)
}

#[async_trait]
impl BaseLLM for ChatBedrock {
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
        let stream = self._stream_messages(&messages, stop).await?;
        let gen_stream = stream.map(|r| r.map(|c| GenerationChunk::new(c.content)));
        Ok(Box::pin(gen_stream))
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
impl ChatModel for ChatBedrock {
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let (system, converse_messages) = convert_messages_to_converse(messages);
        let model = self.config.model.clone().unwrap_or_else(|| self.model.clone());

        let mut body = json!({
            "modelId": model,
            "messages": converse_messages,
        });

        if !system.is_null() {
            body.as_object_mut()
                .unwrap()
                .insert("system".to_string(), system);
        }

        let mut inference_config = json!({});
        if let Some(max_tokens) = self.config.max_tokens {
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("maxTokens".to_string(), json!(max_tokens));
        }
        if let Some(temp) = self.config.temperature {
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("temperature".to_string(), json!(temp));
        }
        if let Some(top_p) = self.config.top_p {
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("topP".to_string(), json!(top_p));
        }
        if let Some(stop_seq) = stop {
            let stop_vec: Vec<String> = stop_seq.iter().map(|&s| s.to_string()).collect();
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("stopSequences".to_string(), json!(stop_vec));
        } else if let Some(stop_seq) = &self.config.stop_sequences {
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("stopSequences".to_string(), json!(stop_seq));
        }

        if !inference_config.as_object().unwrap().is_empty() {
            body.as_object_mut()
                .unwrap()
                .insert("inferenceConfig".to_string(), inference_config);
        }

        let url = format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/converse",
            self.region, model
        );
        let creds = self.credentials();
        let body_bytes = serde_json::to_vec(&body)?;

        let response =
            invoke_bedrock(&self.client, "POST", &url, &self.region, &creds, &body_bytes).await?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            return Err(ChainError::LLMError(format!(
                "Bedrock Converse API error ({}): {}",
                status, err_text
            )));
        }

        let resp: Value = response
            .json()
            .await
            .map_err(|e| ChainError::LLMError(format!("Failed to parse response: {}", e)))?;

        let text = resp["output"]["message"]["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        self._stream_messages(messages, stop).await
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

impl ChatBedrock {
    async fn _stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let (system, converse_messages) = convert_messages_to_converse(messages);
        let model = self.config.model.clone().unwrap_or_else(|| self.model.clone());

        let mut body = json!({
            "modelId": model,
            "messages": converse_messages,
        });

        if !system.is_null() {
            body.as_object_mut()
                .unwrap()
                .insert("system".to_string(), system);
        }

        let mut inference_config = json!({});
        if let Some(max_tokens) = self.config.max_tokens {
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("maxTokens".to_string(), json!(max_tokens));
        }
        if let Some(temp) = self.config.temperature {
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("temperature".to_string(), json!(temp));
        }
        if let Some(top_p) = self.config.top_p {
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("topP".to_string(), json!(top_p));
        }
        if let Some(stop_seq) = stop {
            let stop_vec: Vec<String> = stop_seq.iter().map(|&s| s.to_string()).collect();
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("stopSequences".to_string(), json!(stop_vec));
        } else if let Some(stop_seq) = &self.config.stop_sequences {
            inference_config
                .as_object_mut()
                .unwrap()
                .insert("stopSequences".to_string(), json!(stop_seq));
        }

        if !inference_config.as_object().unwrap().is_empty() {
            body.as_object_mut()
                .unwrap()
                .insert("inferenceConfig".to_string(), inference_config);
        }

        let url = format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/converse-stream",
            self.region, model
        );
        let creds = self.credentials();
        let body_bytes = serde_json::to_vec(&body)?;

        let client = self.client.clone();
        let region = self.region.clone();

        let stream = try_stream! {
            let response = invoke_bedrock(&client, "POST", &url, &region, &creds, &body_bytes).await?;

            if !response.status().is_success() {
                let status = response.status();
                let err_text = response.text().await.unwrap_or_default();
                Err(ChainError::LLMError(format!("Bedrock ConverseStream error ({}): {}", status, err_text)))?;
            }

            let mut byte_stream = response.bytes_stream();
            let mut buffer = Vec::new();

            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk.map_err(|e| ChainError::LLMError(format!("Stream read error: {}", e)))?;
                buffer.extend_from_slice(&chunk);

                let (payloads, consumed) = extract_event_payloads(&buffer);
                if consumed > 0 {
                    buffer.drain(0..consumed);
                }

                for payload in payloads {
                    if let Ok(val) = serde_json::from_slice::<Value>(&payload) {
                        if let Some(delta) = val.get("delta") {
                            if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                                let mut msg = MessageChunk::new(text);
                                if let Some(index) = val.get("contentBlockIndex").and_then(|i| i.as_u64()) {
                                    msg.additional_kwargs.insert("contentBlockIndex".to_string(), json!(index));
                                }
                                yield msg;
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
