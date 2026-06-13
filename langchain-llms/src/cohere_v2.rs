//! Cohere v2 LLM provider.

use async_trait::async_trait;
use futures::stream::BoxStream;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::Result;
use langchain_core::messages::{BaseMessage, MessageType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

use crate::traits::{BaseLLM, ChatModel, FunctionDefinition, ToolDefinition};
use crate::types::{
    Generation, GenerationChunk, GenerationConfig, LLMProvider, LLMResult, MessageChunk,
    ProviderConfig,
};

const BASE_URL: &str = "https://api.cohere.com/v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereV2LLM {
    api_key: String,
    model: String,
    temperature: f64,
    max_tokens: u32,
    base_url: String,
    config: GenerationConfig,
    #[serde(skip)]
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl CohereV2LLM {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        let api_key = api_key.into();
        if api_key.is_empty() {
            warn!("No API key set for Cohere v2");
        }
        Self {
            api_key,
            model: model.into(),
            temperature: 0.7,
            max_tokens: 1024,
            base_url: BASE_URL.to_string(),
            config: GenerationConfig::default(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

impl LLMProvider for CohereV2LLM {
    fn provider_name(&self) -> &'static str {
        "cohere_v2"
    }
}

impl ProviderConfig for CohereV2LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}

#[async_trait]
impl BaseLLM for CohereV2LLM {
    async fn generate(&self, _prompts: &[String], _stop: Option<&[&str]>) -> Result<LLMResult> {
        let text = format!("Cohere v2 stub response for model: {}", self.model);
        Ok(LLMResult {
            generations: vec![vec![Generation {
                text: text.clone(),
                message: Some(BaseMessage::new(text, MessageType::AI)),
                generation_info: None,
            }]],
            llm_output: None,
        })
    }

    async fn stream(
        &self,
        _prompts: &[String],
        _stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>> {
        let text = format!("Cohere v2 stub response for model: {}", self.model);
        let chunks: Vec<Result<GenerationChunk>> = text
            .chars()
            .map(|c| Ok(GenerationChunk::new(c.to_string())))
            .collect();
        Ok(Box::pin(futures::stream::iter(chunks)))
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
impl ChatModel for CohereV2LLM {
    async fn predict_messages(
        &self,
        _messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        _stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let text = format!("Cohere v2 stub response for model: {}", self.model);
        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        _messages: &[BaseMessage],
        _stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let text = format!("Cohere v2 stub response for model: {}", self.model);
        let chunks: Vec<Result<MessageChunk>> = text
            .chars()
            .map(|c| Ok(MessageChunk::new(c.to_string())))
            .collect();
        Ok(Box::pin(futures::stream::iter(chunks)))
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
