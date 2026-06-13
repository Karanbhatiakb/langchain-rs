//! Core traits for LLM providers — [`BaseLLM`], [`ChatModel`], and [`LLMFactory`].

use async_trait::async_trait;
use futures::stream::BoxStream;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::Result;
use langchain_core::messages::BaseMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::{GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

/// Describes a function that an LLM can call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function.
    pub name: String,
    /// A description of what the function does.
    pub description: String,
    /// JSON Schema describing the function parameters.
    pub parameters: Value,
}

/// Describes a tool that an LLM can invoke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The name of the tool.
    pub name: String,
    /// A description of what the tool does.
    pub description: String,
    /// JSON Schema describing the tool parameters.
    pub parameters: Value,
}

/// Base trait for LLM (non-chat) models.
///
/// Provides `generate` for producing text and `stream` for token-by-token
/// streaming.
#[async_trait]
pub trait BaseLLM: Send + Sync {
    /// Generates completions for the given prompts.
    async fn generate(
        &self,
        prompts: &[String],
        stop: Option<&[&str]>,
    ) -> Result<LLMResult>;

    /// Streams generation chunks for the given prompts.
    async fn stream(
        &self,
        prompts: &[String],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<GenerationChunk>>>;

    /// Returns a new instance with the given generation config.
    fn with_config(&self, config: GenerationConfig) -> Arc<dyn BaseLLM>;
    /// Returns a new instance with the given callback manager.
    fn with_callbacks(&self, callbacks: CallbackManager) -> Arc<dyn BaseLLM>;
}

/// Extended trait for chat-oriented LLM models.
///
/// Adds message-based prediction, streaming, function binding, and tool
/// binding on top of [`BaseLLM`].
#[async_trait]
pub trait ChatModel: BaseLLM {
    /// Predicts a response message from a list of messages.
    async fn predict_messages(
        &self,
        messages: &[BaseMessage],
        functions: Option<&[FunctionDefinition]>,
        stop: Option<&[&str]>,
    ) -> Result<BaseMessage>;

    /// Streams a response message as chunks.
    async fn stream_messages(
        &self,
        messages: &[BaseMessage],
        stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>>;

    /// Binds functions to the model for function-calling support.
    fn bind_functions(&self, functions: Vec<FunctionDefinition>) -> Arc<dyn ChatModel>;
    /// Binds tools to the model for tool-calling support.
    fn bind_tools(&self, tools: Vec<ToolDefinition>) -> Arc<dyn ChatModel>;
}

/// Factory trait for creating chat model instances from configuration maps.
pub trait LLMFactory: Send + Sync {
    /// Creates a chat model from a config map.
    fn create(config: &HashMap<String, Value>) -> Result<Arc<dyn ChatModel>>;
    /// Lists available model names for this provider.
    fn list_models() -> Vec<String>;
}
