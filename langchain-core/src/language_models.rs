//! Language model abstractions — base traits, chat models, LLMs, and fakes.
//!
//! This module mirrors the Python `langchain_core.language_models` package and
//! provides three layers of abstraction:
//!
//! - [`BaseLanguageModel`] — the most generic interface shared by **all** language
//!   models (chat *and* text-in/text-out).
//! - [`BaseChatModel`] — extends `BaseLanguageModel` for models that consume and
//!   produce chat messages.
//! - [`BaseLLM`] — extends `BaseLanguageModel` for legacy text-in/text-out models.
//!
//! Concrete helpers for testing are also included:
//! - [`SimpleChatModel`] — convenience base for chat models that only need to
//!   implement a synchronous `_call`.
//! - [`FakeListChatModel`] / [`FakeListLLM`] / [`DeterministicFakeListChatModel`] —
//!   deterministic fakes that cycle through a predetermined list of responses.

use crate::errors::*;
use crate::messages::{BaseMessage, MessageType};
use crate::schemas::{
    ChatGeneration, ChatResult, Generation, LLMResult, MessageChunk,
};
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// PromptValue — unified prompt representation
// ---------------------------------------------------------------------------

/// A language-model-agnostic prompt value.
///
/// Mirrors Python's `langchain_core.prompt_values.PromptValue`. A prompt can
/// either be a plain string (for text-in/text-out LLMs) or a list of chat
/// messages (for chat models).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptValue {
    /// A plain-text prompt.
    Text(String),
    /// A chat prompt consisting of an ordered list of messages.
    Messages(Vec<BaseMessage>),
}

impl PromptValue {
    /// Returns the prompt as a plain-text string.
    ///
    /// For [`PromptValue::Text`] the inner string is returned verbatim.
    /// For [`PromptValue::Messages`] the messages are concatenated
    /// (role-prefixed) into a single string.
    pub fn to_string(&self) -> String {
        match self {
            PromptValue::Text(s) => s.clone(),
            PromptValue::Messages(msgs) => msgs
                .iter()
                .map(|m| {
                    let role = match m.message_type {
                        MessageType::Human => "Human",
                        MessageType::AI => "AI",
                        MessageType::System => "System",
                        MessageType::Tool => "Tool",
                        MessageType::Function => "Function",
                        MessageType::Generic => "Generic",
                        MessageType::Chat => "Chat",
                    };
                    format!("{}: {}", role, m.content)
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    /// Returns the messages, converting a text prompt into a single
    /// [`HumanMessage`](crate::messages::HumanMessage) if necessary.
    pub fn to_messages(&self) -> Vec<BaseMessage> {
        match self {
            PromptValue::Text(s) => {
                vec![BaseMessage::new(s.clone(), MessageType::Human)]
            }
            PromptValue::Messages(msgs) => msgs.clone(),
        }
    }
}

impl From<String> for PromptValue {
    fn from(s: String) -> Self {
        PromptValue::Text(s)
    }
}

impl From<&str> for PromptValue {
    fn from(s: &str) -> Self {
        PromptValue::Text(s.to_string())
    }
}

impl From<Vec<BaseMessage>> for PromptValue {
    fn from(msgs: Vec<BaseMessage>) -> Self {
        PromptValue::Messages(msgs)
    }
}

// ---------------------------------------------------------------------------
// ChatGenerationChunk
// ---------------------------------------------------------------------------

/// A streaming chunk of a chat generation.
///
/// Unlike [`ChatGeneration`], which carries a complete message, a
/// `ChatGenerationChunk` represents a partial fragment produced during
/// streaming. Multiple chunks are typically merged to reconstruct the full
/// generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGenerationChunk {
    /// The partial message for this chunk.
    pub message: MessageChunk,
    /// Optional provider-specific metadata for this chunk.
    pub generation_info: Option<HashMap<String, serde_json::Value>>,
}

impl ChatGenerationChunk {
    /// Creates a new `ChatGenerationChunk` from a [`MessageChunk`].
    pub fn new(message: MessageChunk) -> Self {
        Self {
            message,
            generation_info: None,
        }
    }

    /// Adds generation info (builder pattern).
    pub fn with_generation_info(
        mut self,
        info: HashMap<String, serde_json::Value>,
    ) -> Self {
        self.generation_info = Some(info);
        self
    }

    /// Merges another chunk into this one, concatenating content and
    /// merging additional kwargs.
    pub fn merge(&mut self, other: ChatGenerationChunk) {
        self.message.content.push_str(&other.message.content);
        for (k, v) in other.message.additional_kwargs {
            self.message.additional_kwargs.insert(k, v);
        }
        if other.generation_info.is_some() {
            self.generation_info = other.generation_info;
        }
    }
}

// ---------------------------------------------------------------------------
// ModelProfile
// ---------------------------------------------------------------------------

/// Metadata describing a model's capabilities and constraints.
///
/// Mirrors Python's `langchain_core.language_models.model_profile.ModelProfile`.
/// All fields are optional (`pub … Option<…>`) so that partial profiles can be
/// represented without sentinel values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    /// Human-readable model name (e.g. `"GPT-4o"`).
    pub name: Option<String>,
    /// Model lifecycle status (e.g. `"active"`, `"deprecated"`).
    pub status: Option<String>,
    /// Release date in ISO 8601 format.
    pub release_date: Option<String>,
    /// Date the model was last updated (ISO 8601).
    pub last_updated: Option<String>,
    /// Whether the model weights are openly available.
    pub open_weights: Option<bool>,
    /// Maximum context window in tokens.
    pub max_input_tokens: Option<usize>,
    /// Whether text inputs are supported.
    pub text_inputs: Option<bool>,
    /// Whether image inputs are supported.
    pub image_inputs: Option<bool>,
    /// Whether image-URL inputs are supported.
    pub image_url_inputs: Option<bool>,
    /// Whether PDF inputs are supported.
    pub pdf_inputs: Option<bool>,
    /// Whether audio inputs are supported.
    pub audio_inputs: Option<bool>,
    /// Whether video inputs are supported.
    pub video_inputs: Option<bool>,
    /// Whether images can be included in tool-message content.
    pub image_tool_message: Option<bool>,
    /// Whether PDFs can be included in tool-message content.
    pub pdf_tool_message: Option<bool>,
    /// Maximum output tokens.
    pub max_output_tokens: Option<usize>,
    /// Whether the model supports reasoning / chain-of-thought output.
    pub reasoning_output: Option<bool>,
    /// Whether text outputs are supported.
    pub text_outputs: Option<bool>,
    /// Whether image outputs are supported.
    pub image_outputs: Option<bool>,
    /// Whether audio outputs are supported.
    pub audio_outputs: Option<bool>,
    /// Whether video outputs are supported.
    pub video_outputs: Option<bool>,
    /// Whether the model supports tool calling.
    pub tool_calling: Option<bool>,
    /// Whether the model supports tool choice.
    pub tool_choice: Option<bool>,
    /// Whether the model supports native structured output.
    pub structured_output: Option<bool>,
    /// Whether the model supports file attachments.
    pub attachment: Option<bool>,
    /// Whether the model supports a temperature parameter.
    pub temperature: Option<bool>,
}

impl ModelProfile {
    /// Creates an empty `ModelProfile` with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            name: None,
            status: None,
            release_date: None,
            last_updated: None,
            open_weights: None,
            max_input_tokens: None,
            text_inputs: None,
            image_inputs: None,
            image_url_inputs: None,
            pdf_inputs: None,
            audio_inputs: None,
            video_inputs: None,
            image_tool_message: None,
            pdf_tool_message: None,
            max_output_tokens: None,
            reasoning_output: None,
            text_outputs: None,
            image_outputs: None,
            audio_outputs: None,
            video_outputs: None,
            tool_calling: None,
            tool_choice: None,
            structured_output: None,
            attachment: None,
            temperature: None,
        }
    }

    /// Returns the context window size (`max_input_tokens`) if set.
    pub fn context_window(&self) -> Option<usize> {
        self.max_input_tokens
    }
}

impl Default for ModelProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// A registry mapping model identifiers to their [`ModelProfile`]s.
pub type ModelProfileRegistry = HashMap<String, ModelProfile>;

// ---------------------------------------------------------------------------
// LangSmithParams
// ---------------------------------------------------------------------------

/// Parameters emitted for LangSmith-style tracing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSmithParams {
    /// Provider of the model (e.g. `"openai"`).
    pub ls_provider: String,
    /// Name of the model (e.g. `"gpt-4o"`).
    pub ls_model_name: String,
    /// Model type: `"chat"` or `"llm"`.
    pub ls_model_type: String,
    /// Temperature used for generation.
    pub ls_temperature: Option<f64>,
    /// Maximum tokens for generation.
    pub ls_max_tokens: Option<usize>,
    /// Stop words for generation.
    pub ls_stop: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// BaseLanguageModel
// ---------------------------------------------------------------------------

/// The most generic interface shared by **all** language models.
///
/// Mirrors Python's `langchain_core.language_models.base.BaseLanguageModel`.
/// Both [`BaseChatModel`] and [`BaseLLM`] extend this trait.
///
/// # Type parameter
/// `O` — the primary output type (`BaseMessage` for chat models, `String` for
/// text LLMs).
#[async_trait]
pub trait BaseLanguageModel<O: Send + 'static>: Send + Sync + 'static {
    /// Pass a sequence of [`PromptValue`]s to the model and return an
    /// [`LLMResult`].
    ///
    /// This is the model-agnostic entry point: the model converts each prompt
    /// value into whatever representation it needs internally.
    async fn generate_prompt(
        &self,
        prompts: Vec<PromptValue>,
        stop: Option<Vec<String>>,
    ) -> Result<LLMResult>;

    /// Returns the number of tokens in `text`.
    ///
    /// The default implementation provides a rough whitespace-based estimate.
    /// Model-specific implementations should override this with an accurate
    /// tokenizer.
    fn get_num_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Returns the token IDs for `text`.
    ///
    /// The default returns an empty vector; concrete implementations should
    /// override this with a model-specific tokenizer.
    fn get_token_ids(&self, _text: &str) -> Vec<usize> {
        Vec::new()
    }

    /// Returns a JSON value describing the invocation parameters of this model
    /// (e.g. model name, temperature, max tokens).
    fn invocation_params(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    /// Returns a string identifying the model type (e.g. `"openai-chat"`,
    /// `"fake-list"`).
    fn _type(&self) -> &str;

    /// Returns LangSmith tracing parameters.
    ///
    /// The default implementation derives the provider name from `_type()`.
    fn get_ls_params(&self) -> LangSmithParams {
        LangSmithParams {
            ls_provider: self._type().to_string(),
            ls_model_name: String::new(),
            ls_model_type: String::new(),
            ls_temperature: None,
            ls_max_tokens: None,
            ls_stop: None,
        }
    }
}

// ---------------------------------------------------------------------------
// BaseChatModel
// ---------------------------------------------------------------------------

/// Base trait for chat-oriented language models.
///
/// Mirrors Python's `langchain_core.language_models.chat_models.BaseChatModel`.
/// Implementations receive a list of [`BaseMessage`]s and return a
/// [`ChatResult`].
///
/// At a minimum, implementors must provide [`generate`](BaseChatModel::generate).
/// The `predict`, `predict_messages`, and `stream` methods have default
/// implementations built on top of `generate`.
#[async_trait]
pub trait BaseChatModel: BaseLanguageModel<BaseMessage> {
    /// Generate a chat result from one or more message sequences.
    ///
    /// Each element of `messages` is a separate conversation (prompt). The
    /// outer `Vec` allows batched generation.
    async fn generate(
        &self,
        messages: Vec<Vec<BaseMessage>>,
        stop: Option<Vec<String>>,
    ) -> Result<ChatResult>;

    /// Convenience wrapper: generate for a single conversation and return the
    /// assistant message.
    async fn predict(
        &self,
        messages: Vec<BaseMessage>,
        stop: Option<Vec<String>>,
    ) -> Result<BaseMessage> {
        let result = self.generate(vec![messages], stop).await?;
        let gen = result
            .generations
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::LLMError("No generations returned".into()))?;
        Ok(gen.message)
    }

    /// Convenience wrapper: generate for a single conversation and return the
    /// [`ChatGeneration`] (includes `generation_info`).
    async fn predict_messages(
        &self,
        messages: Vec<BaseMessage>,
        stop: Option<Vec<String>>,
    ) -> Result<ChatGeneration> {
        let result = self.generate(vec![messages], stop).await?;
        result
            .generations
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::LLMError("No generations returned".into()))
    }

    /// Stream partial chat-generation chunks for a single conversation.
    ///
    /// The default implementation returns a single-element stream containing
    /// the full generation (i.e. no incremental streaming).
    async fn stream(
        &self,
        messages: Vec<BaseMessage>,
        stop: Option<Vec<String>>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<ChatGenerationChunk>> + Send>>>
    {
        let result = self.generate(vec![messages], stop).await?;
        let gen = result
            .generations
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::LLMError("No generations returned".into()))?;
        let chunk = ChatGenerationChunk::new(MessageChunk::new(
            gen.message.content.clone(),
            gen.message.message_type.clone(),
        ));
        let stream = futures::stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    /// Returns a pinned, boxed stream of chat-generation chunks.
    ///
    /// This is a convenience that calls [`stream`](BaseChatModel::stream) and
    /// wraps the result in a [`BoxStream`].
    async fn stream_boxed(
        &self,
        messages: Vec<BaseMessage>,
        stop: Option<Vec<String>>,
    ) -> Result<BoxStream<'static, Result<ChatGenerationChunk>>> {
        self.stream(messages, stop).await
    }
}

// ---------------------------------------------------------------------------
// BaseLLM
// ---------------------------------------------------------------------------

/// Base trait for text-in / text-out language models.
///
/// Mirrors Python's `langchain_core.language_models.llms.BaseLLM`.
/// Implementations receive string prompts and return [`LLMResult`].
#[async_trait]
pub trait BaseLLM: BaseLanguageModel<String> {
    /// Generate completions for a batch of string prompts.
    async fn generate(
        &self,
        prompts: Vec<String>,
        stop: Option<Vec<String>>,
    ) -> Result<LLMResult>;

    /// Convenience wrapper: generate for a single prompt and return the
    /// top-generation text.
    async fn predict(
        &self,
        text: &str,
        stop: Option<Vec<String>>,
    ) -> Result<String> {
        let result = self.generate(vec![text.to_string()], stop).await?;
        let gens = result
            .generations
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::LLMError("No generations returned".into()))?;
        let gen = gens
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::LLMError("No generation in result".into()))?;
        Ok(gen.text)
    }
}

// ---------------------------------------------------------------------------
// SimpleChatModel — helper for implementing BaseChatModel
// ---------------------------------------------------------------------------

/// A simplified base for chat models that only need to implement a synchronous
/// `_call` method returning a string.
///
/// Mirrors Python's `langchain_core.language_models.chat_models.SimpleChatModel`.
/// The [`generate`](SimpleChatModel::generate) implementation wraps the
/// string output in an AI [`BaseMessage`] / [`ChatGeneration`].
///
/// Implementors must provide [`_call`](SimpleChatModel::_call) and
/// [`_type`](BaseLanguageModel::_type).
#[async_trait]
pub trait SimpleChatModel: BaseChatModel + Send + Sync + 'static {
    /// Synchronous core logic: given a list of messages, return a string
    /// response.
    fn _call(
        &self,
        messages: Vec<BaseMessage>,
        stop: Option<Vec<String>>,
    ) -> Result<String>;

    /// Default `generate` implementation: calls `_call` and wraps the result
    /// in a `ChatResult`.
    fn _generate(
        &self,
        messages: Vec<BaseMessage>,
        stop: Option<Vec<String>>,
    ) -> Result<ChatResult> {
        let output = self._call(messages, stop)?;
        let message = BaseMessage::new(output, MessageType::AI);
        let generation = ChatGeneration::new(message);
        Ok(ChatResult {
            generations: vec![generation],
            llm_output: None,
        })
    }
}

/// Blanket async `generate` for all `SimpleChatModel` implementors.
///
/// Because Rust traits do not support default async method bodies that call
/// sync trait methods, we provide a free function that any `SimpleChatModel`
/// implementor can delegate to.
pub async fn simple_chat_model_generate<S: SimpleChatModel + ?Sized>(
    model: &S,
    messages: Vec<Vec<BaseMessage>>,
    stop: Option<Vec<String>>,
) -> Result<ChatResult> {
    if messages.is_empty() {
        return Err(ChainError::LLMError("No messages provided".into()));
    }
    model._generate(
        messages.into_iter().next().ok_or_else(|| {
            ChainError::LLMError("Failed to extract message batch".into())
        })?,
        stop,
    )
}

// ---------------------------------------------------------------------------
// FakeListChatModel
// ---------------------------------------------------------------------------

/// A fake chat model for testing that cycles through a predetermined list of
/// string responses.
///
/// Mirrors Python's `FakeListChatModel`. Each call to `generate` returns the
/// next response in the list, wrapping back to the start when the end is
/// reached.
pub struct FakeListChatModel {
    /// The predetermined responses to cycle through.
    pub responses: Vec<String>,
    /// An optional sleep duration (milliseconds) simulated between responses.
    pub sleep_ms: Option<u64>,
    counter: Arc<AtomicUsize>,
}

impl FakeListChatModel {
    /// Creates a new `FakeListChatModel` with the given responses.
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            sleep_ms: None,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Sets the simulated sleep duration in milliseconds (builder pattern).
    pub fn with_sleep_ms(mut self, ms: u64) -> Self {
        self.sleep_ms = Some(ms);
        self
    }

    fn next_response(&self) -> Result<String> {
        if self.responses.is_empty() {
            return Err(ChainError::LLMError(
                "FakeListChatModel has no responses".into(),
            ));
        }
        let idx = self.counter.fetch_add(1, Ordering::SeqCst);
        let response = self
            .responses
            .get(idx % self.responses.len())
            .ok_or_else(|| ChainError::LLMError("Index out of bounds".into()))?;
        Ok(response.clone())
    }
}

#[async_trait]
impl BaseLanguageModel<BaseMessage> for FakeListChatModel {
    async fn generate_prompt(
        &self,
        prompts: Vec<PromptValue>,
        stop: Option<Vec<String>>,
    ) -> Result<LLMResult> {
        let mut all_generations = Vec::new();
        for prompt in prompts {
            let messages = prompt.to_messages();
            let chat_result = self.generate(vec![messages], stop.clone()).await?;
            let gen_text = chat_result
                .generations
                .first()
                .map(|g| g.text.clone())
                .ok_or_else(|| ChainError::LLMError("No chat generation".into()))?;
            all_generations.push(vec![Generation::new(gen_text)]);
        }
        Ok(LLMResult::new(all_generations))
    }

    fn get_num_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    fn _type(&self) -> &str {
        "fake-list-chat-model"
    }
}

#[async_trait]
impl BaseChatModel for FakeListChatModel {
    async fn generate(
        &self,
        messages: Vec<Vec<BaseMessage>>,
        stop: Option<Vec<String>>,
    ) -> Result<ChatResult> {
        let _ = messages;
        let _ = stop;
        if let Some(ms) = self.sleep_ms {
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        }
        let response = self.next_response()?;
        let message = BaseMessage::new(response, MessageType::AI);
        let generation = ChatGeneration::new(message);
        Ok(ChatResult {
            generations: vec![generation],
            llm_output: None,
        })
    }

    async fn stream(
        &self,
        messages: Vec<BaseMessage>,
        stop: Option<Vec<String>>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<ChatGenerationChunk>> + Send>>>
    {
        let result = self.generate(vec![messages], stop).await?;
        let gen = result
            .generations
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::LLMError("No generations returned".into()))?;
        let response_text = gen.message.content.clone();
        let chunks: Vec<Result<ChatGenerationChunk>> = response_text
            .chars()
            .map(|c| {
                Ok(ChatGenerationChunk::new(MessageChunk::new(
                    c.to_string(),
                    MessageType::AI,
                )))
            })
            .collect();
        let stream = futures::stream::iter(chunks);
        Ok(Box::pin(stream))
    }
}

#[async_trait]
impl SimpleChatModel for FakeListChatModel {
    fn _call(
        &self,
        _messages: Vec<BaseMessage>,
        _stop: Option<Vec<String>>,
    ) -> Result<String> {
        self.next_response()
    }
}

// ---------------------------------------------------------------------------
// FakeListLLM
// ---------------------------------------------------------------------------

/// A fake text-in/text-out LLM for testing that cycles through a
/// predetermined list of responses.
///
/// Mirrors Python's `FakeListLLM`. Each call to `generate` returns the next
/// response in the list, wrapping back to the start when the end is reached.
pub struct FakeListLLM {
    /// The predetermined responses to cycle through.
    pub responses: Vec<String>,
    /// An optional sleep duration (milliseconds) simulated between responses.
    pub sleep_ms: Option<u64>,
    counter: Arc<AtomicUsize>,
}

impl FakeListLLM {
    /// Creates a new `FakeListLLM` with the given responses.
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            sleep_ms: None,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Sets the simulated sleep duration in milliseconds (builder pattern).
    pub fn with_sleep_ms(mut self, ms: u64) -> Self {
        self.sleep_ms = Some(ms);
        self
    }

    fn next_response(&self) -> Result<String> {
        if self.responses.is_empty() {
            return Err(ChainError::LLMError(
                "FakeListLLM has no responses".into(),
            ));
        }
        let idx = self.counter.fetch_add(1, Ordering::SeqCst);
        let response = self
            .responses
            .get(idx % self.responses.len())
            .ok_or_else(|| ChainError::LLMError("Index out of bounds".into()))?;
        Ok(response.clone())
    }
}

#[async_trait]
impl BaseLanguageModel<String> for FakeListLLM {
    async fn generate_prompt(
        &self,
        prompts: Vec<PromptValue>,
        stop: Option<Vec<String>>,
    ) -> Result<LLMResult> {
        let prompt_strings: Vec<String> =
            prompts.iter().map(|p| p.to_string()).collect();
        self.generate(prompt_strings, stop).await
    }

    fn get_num_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    fn _type(&self) -> &str {
        "fake-list-llm"
    }
}

#[async_trait]
impl BaseLLM for FakeListLLM {
    async fn generate(
        &self,
        prompts: Vec<String>,
        stop: Option<Vec<String>>,
    ) -> Result<LLMResult> {
        let _ = stop;
        if let Some(ms) = self.sleep_ms {
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        }
        let mut generations = Vec::with_capacity(prompts.len());
        for _ in &prompts {
            let response = self.next_response()?;
            generations.push(vec![Generation::new(response)]);
        }
        Ok(LLMResult::new(generations))
    }
}

// ---------------------------------------------------------------------------
// DeterministicFakeListChatModel
// ---------------------------------------------------------------------------

/// A deterministic fake chat model for testing that always returns the same
/// response for a given input index.
///
/// Unlike [`FakeListChatModel`], which advances a global counter on every
/// call, this model uses the **index of the input prompt** (i.e. the position
/// of the message batch within the `generate` call) to select the response.
/// This makes the output fully deterministic with respect to the call
/// structure, which is useful for reproducible test assertions.
pub struct DeterministicFakeListChatModel {
    /// The predetermined responses. Input index `i` selects
    /// `responses[i % responses.len()]`.
    pub responses: Vec<String>,
}

impl DeterministicFakeListChatModel {
    /// Creates a new `DeterministicFakeListChatModel` with the given
    /// responses.
    pub fn new(responses: Vec<String>) -> Self {
        Self { responses }
    }

    fn response_at(&self, index: usize) -> Result<String> {
        if self.responses.is_empty() {
            return Err(ChainError::LLMError(
                "DeterministicFakeListChatModel has no responses".into(),
            ));
        }
        let response = self
            .responses
            .get(index % self.responses.len())
            .ok_or_else(|| ChainError::LLMError("Index out of bounds".into()))?;
        Ok(response.clone())
    }
}

#[async_trait]
impl BaseLanguageModel<BaseMessage> for DeterministicFakeListChatModel {
    async fn generate_prompt(
        &self,
        prompts: Vec<PromptValue>,
        stop: Option<Vec<String>>,
    ) -> Result<LLMResult> {
        let mut all_generations = Vec::with_capacity(prompts.len());
        for (i, prompt) in prompts.into_iter().enumerate() {
            let messages = prompt.to_messages();
            let chat_result = self.generate(vec![messages], stop.clone()).await?;
            let gen_text = chat_result
                .generations
                .first()
                .map(|g| g.text.clone())
                .ok_or_else(|| ChainError::LLMError("No chat generation".into()))?;
            all_generations.push(vec![Generation::new(gen_text)]);
            let _ = i;
        }
        Ok(LLMResult::new(all_generations))
    }

    fn get_num_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    fn _type(&self) -> &str {
        "deterministic-fake-list-chat-model"
    }
}

#[async_trait]
impl BaseChatModel for DeterministicFakeListChatModel {
    async fn generate(
        &self,
        messages: Vec<Vec<BaseMessage>>,
        stop: Option<Vec<String>>,
    ) -> Result<ChatResult> {
        let _ = messages;
        let _ = stop;
        let response = self.response_at(0)?;
        let message = BaseMessage::new(response, MessageType::AI);
        let generation = ChatGeneration::new(message);
        Ok(ChatResult {
            generations: vec![generation],
            llm_output: None,
        })
    }

    async fn predict(
        &self,
        messages: Vec<BaseMessage>,
        stop: Option<Vec<String>>,
    ) -> Result<BaseMessage> {
        let _ = messages;
        let _ = stop;
        let response = self.response_at(0)?;
        Ok(BaseMessage::new(response, MessageType::AI))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_value_text() {
        let pv = PromptValue::Text("hello".to_string());
        assert_eq!(pv.to_string(), "hello");
        let msgs = pv.to_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "hello");
    }

    #[tokio::test]
    async fn test_prompt_value_messages() {
        let msgs = vec![
            BaseMessage::new("Hi", MessageType::Human),
            BaseMessage::new("Hello", MessageType::AI),
        ];
        let pv = PromptValue::Messages(msgs);
        let text = pv.to_string();
        assert!(text.contains("Human: Hi"));
        assert!(text.contains("AI: Hello"));
        let msgs_out = pv.to_messages();
        assert_eq!(msgs_out.len(), 2);
    }

    #[tokio::test]
    async fn test_fake_list_chat_model() {
        let model = FakeListChatModel::new(vec![
            "response1".to_string(),
            "response2".to_string(),
        ]);
        assert_eq!(model._type(), "fake-list-chat-model");

        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let result = model
            .generate(vec![msgs.clone()], None)
            .await
            .expect("generate failed");
        assert_eq!(result.generations[0].message.content, "response1");

        let result = model
            .generate(vec![msgs.clone()], None)
            .await
            .expect("generate failed");
        assert_eq!(result.generations[0].message.content, "response2");

        let result = model
            .generate(vec![msgs], None)
            .await
            .expect("generate failed");
        assert_eq!(result.generations[0].message.content, "response1");
    }

    #[tokio::test]
    async fn test_fake_list_chat_model_predict() {
        let model = FakeListChatModel::new(vec!["hello".to_string()]);
        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let msg = model.predict(msgs, None).await.expect("predict failed");
        assert_eq!(msg.content, "hello");
        assert!(matches!(msg.message_type, MessageType::AI));
    }

    #[tokio::test]
    async fn test_fake_list_llm() {
        let model = FakeListLLM::new(vec![
            "answer1".to_string(),
            "answer2".to_string(),
        ]);
        assert_eq!(model._type(), "fake-list-llm");

        let result = model
            .generate(vec!["prompt1".to_string()], None)
            .await
            .expect("generate failed");
        assert_eq!(result.generations[0][0].text, "answer1");

        let result = model
            .generate(vec!["prompt2".to_string()], None)
            .await
            .expect("generate failed");
        assert_eq!(result.generations[0][0].text, "answer2");

        let result = model
            .generate(vec!["prompt3".to_string()], None)
            .await
            .expect("generate failed");
        assert_eq!(result.generations[0][0].text, "answer1");
    }

    #[tokio::test]
    async fn test_fake_list_llm_predict() {
        let model = FakeListLLM::new(vec!["world".to_string()]);
        let text = model.predict("hello", None).await.expect("predict failed");
        assert_eq!(text, "world");
    }

    #[tokio::test]
    async fn test_deterministic_fake_list_chat_model() {
        let model = DeterministicFakeListChatModel::new(vec![
            "first".to_string(),
            "second".to_string(),
        ]);
        assert_eq!(model._type(), "deterministic-fake-list-chat-model");

        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let result = model.predict(msgs, None).await.expect("predict failed");
        assert_eq!(result.content, "first");
    }

    #[tokio::test]
    async fn test_model_profile() {
        let profile = ModelProfile::new();
        assert!(profile.name.is_none());
        assert!(profile.context_window().is_none());

        let profile = ModelProfile {
            name: Some("GPT-4o".to_string()),
            max_input_tokens: Some(128000),
            text_inputs: Some(true),
            ..ModelProfile::new()
        };
        assert_eq!(profile.name.as_deref(), Some("GPT-4o"));
        assert_eq!(profile.context_window(), Some(128000));
    }

    #[tokio::test]
    async fn test_chat_generation_chunk_merge() {
        let mut chunk1 = ChatGenerationChunk::new(MessageChunk::new(
            "Hello".to_string(),
            MessageType::AI,
        ));
        let chunk2 = ChatGenerationChunk::new(MessageChunk::new(
            " world".to_string(),
            MessageType::AI,
        ));
        chunk1.merge(chunk2);
        assert_eq!(chunk1.message.content, "Hello world");
    }

    #[tokio::test]
    async fn test_prompt_value_from_impls() {
        let pv: PromptValue = "hello".into();
        assert!(matches!(pv, PromptValue::Text(_)));

        let pv: PromptValue = String::from("hello").into();
        assert!(matches!(pv, PromptValue::Text(_)));

        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let pv: PromptValue = msgs.into();
        assert!(matches!(pv, PromptValue::Messages(_)));
    }

    #[tokio::test]
    async fn test_lang_smith_params() {
        let model = FakeListChatModel::new(vec!["hi".to_string()]);
        let params = model.get_ls_params();
        assert_eq!(params.ls_provider, "fake-list-chat-model");
    }

    #[tokio::test]
    async fn test_get_num_tokens_default() {
        let model = FakeListLLM::new(vec!["hi".to_string()]);
        assert_eq!(model.get_num_tokens("hello world foo"), 3);
        assert_eq!(model.get_num_tokens(""), 0);
    }

    #[tokio::test]
    async fn test_get_token_ids_default() {
        let model = FakeListLLM::new(vec!["hi".to_string()]);
        let ids = model.get_token_ids("hello world");
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn test_invocation_params_default() {
        let model = FakeListLLM::new(vec!["hi".to_string()]);
        assert!(model.invocation_params().is_null());
    }

    #[tokio::test]
    async fn test_fake_list_chat_model_empty_responses_error() {
        let model = FakeListChatModel::new(vec![]);
        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let result = model.generate(vec![msgs], None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fake_list_chat_model_with_sleep() {
        let model = FakeListChatModel::new(vec!["slow".to_string()]).with_sleep_ms(5);
        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let start = std::time::Instant::now();
        let result = model.generate(vec![msgs], None).await;
        let elapsed = start.elapsed();
        assert!(result.is_ok());
        assert!(elapsed.as_millis() >= 5);
    }

    #[tokio::test]
    async fn test_fake_list_llm_empty_responses_error() {
        let model = FakeListLLM::new(vec![]);
        let result = model.generate(vec!["hello".into()], None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fake_list_llm_generate_multiple_prompts() {
        let model = FakeListLLM::new(vec!["ans1".into(), "ans2".into(), "ans3".into()]);
        let result = model.generate(vec!["q1".into(), "q2".into(), "q3".into()], None).await.unwrap();
        assert_eq!(result.generations.len(), 3);
        assert_eq!(result.generations[0][0].text, "ans1");
        assert_eq!(result.generations[1][0].text, "ans2");
        assert_eq!(result.generations[2][0].text, "ans3");
    }

    #[tokio::test]
    async fn test_prompt_value_from_str() {
        let pv: PromptValue = "hello world".into();
        assert_eq!(pv.to_string(), "hello world");
    }

    #[tokio::test]
    async fn test_chat_generation_chunk_with_info() {
        let mut info = HashMap::new();
        info.insert("finish_reason".into(), serde_json::json!("stop"));
        let chunk = ChatGenerationChunk::new(MessageChunk::new("Hello".to_string(), MessageType::AI))
            .with_generation_info(info);
        assert!(chunk.generation_info.is_some());
    }

    #[tokio::test]
    async fn test_chat_generation_chunk_merge_with_info() {
        let mut info = HashMap::new();
        info.insert("a".into(), serde_json::json!(1));
        let mut chunk1 = ChatGenerationChunk::new(MessageChunk::new("Hello".to_string(), MessageType::AI))
            .with_generation_info(info);
        let chunk2 = ChatGenerationChunk::new(MessageChunk::new(" world".to_string(), MessageType::AI));
        chunk1.merge(chunk2);
        assert_eq!(chunk1.message.content, "Hello world");
    }

    #[tokio::test]
    async fn test_deterministic_fake_no_responses_error() {
        let model = DeterministicFakeListChatModel::new(vec![]);
        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let result = model.predict(msgs, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_simple_chat_model_generate_empty() {
        struct DummyModel;
        #[async_trait]
        impl BaseChatModel for DummyModel {
            async fn generate(&self, _: Vec<Vec<BaseMessage>>, _: Option<Vec<String>>) -> Result<ChatResult> {
                Err(ChainError::LLMError("not implemented".into()))
            }
        }
        #[async_trait]
        impl BaseLanguageModel<BaseMessage> for DummyModel {
            async fn generate_prompt(&self, _: Vec<PromptValue>, _: Option<Vec<String>>) -> Result<LLMResult> {
                Err(ChainError::LLMError("not implemented".into()))
            }
            fn _type(&self) -> &str { "dummy" }
        }
        let _ = DummyModel;
    }

    #[tokio::test]
    async fn test_fake_list_chat_model_stream_single_char() {
        let model = FakeListChatModel::new(vec!["a".to_string()]);
        let msgs = vec![BaseMessage::new("Hi", MessageType::Human)];
        let mut stream = model.stream(msgs, None).await.unwrap();
        use futures::StreamExt;
        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk.message.content, "a");
    }

    #[tokio::test]
    async fn test_model_profile_builder() {
        let profile = ModelProfile::new();
        assert_eq!(profile.context_window(), None);
        let profile = ModelProfile {
            name: Some("GPT-4".into()),
            max_input_tokens: Some(8192),
            ..ModelProfile::new()
        };
        assert_eq!(profile.name.as_deref(), Some("GPT-4"));
        assert_eq!(profile.context_window(), Some(8192));
    }

    #[test]
    fn test_send_sync_language_model_types() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FakeListChatModel>();
        assert_sync::<FakeListChatModel>();
        assert_send::<FakeListLLM>();
        assert_sync::<FakeListLLM>();
        assert_send::<DeterministicFakeListChatModel>();
        assert_sync::<DeterministicFakeListChatModel>();
        assert_send::<PromptValue>();
        assert_sync::<PromptValue>();
        assert_send::<ModelProfile>();
        assert_sync::<ModelProfile>();
        assert_send::<ChatGenerationChunk>();
        assert_sync::<ChatGenerationChunk>();
    }
}
