use async_trait::async_trait;
use futures::stream::BoxStream;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::Result;
use langchain_core::messages::{BaseMessage, MessageType};
use std::sync::Arc;
use tracing::warn;

use crate::traits::{BaseLLM, ChatModel, FunctionDefinition, ToolDefinition};
use crate::types::{Generation, GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

pub struct Gpt4AllLLM {
    model: String,
    api_key: String,
    config: GenerationConfig,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl std::fmt::Debug for Gpt4AllLLM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gpt4AllLLM")
            .field("model", &self.model)
            .finish()
    }
}

impl Gpt4AllLLM {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        let _api_key = api_key.into();
        warn!("GPT4All is a local model — no remote API key needed");
        Self {
            model: model.into(),
            api_key: String::new(),
            config: GenerationConfig::default(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }
}

impl Clone for Gpt4AllLLM {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            config: self.config.clone(),
            callbacks: self.callbacks.clone(),
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for Gpt4AllLLM {
    async fn generate(&self, _prompts: &[String], _stop: Option<&[&str]>) -> Result<LLMResult> {
        let text = format!("GPT4All stub response for model: {}", self.model);
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
        let text = format!("GPT4All stub response for model: {}", self.model);
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
impl ChatModel for Gpt4AllLLM {
    async fn predict_messages(
        &self,
        _messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        _stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let text = format!("GPT4All stub response for model: {}", self.model);
        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        _messages: &[BaseMessage],
        _stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let text = format!("GPT4All stub response for model: {}", self.model);
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
