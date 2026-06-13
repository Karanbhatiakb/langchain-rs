//! Fake (testing) LLM provider — returns predetermined responses.

use async_trait::async_trait;
use futures::stream::BoxStream;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::Result;
use langchain_core::messages::{BaseMessage, MessageType};
use std::sync::Arc;

use crate::traits::{BaseLLM, ChatModel, FunctionDefinition, ToolDefinition};
use crate::types::{Generation, GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

pub struct FakeListLLM {
    responses: Vec<String>,
    config: GenerationConfig,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl FakeListLLM {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            config: GenerationConfig::default(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    fn get_response(&self) -> String {
        self.responses
            .first()
            .cloned()
            .unwrap_or_default()
    }
}

impl Clone for FakeListLLM {
    fn clone(&self) -> Self {
        Self {
            responses: self.responses.clone(),
            config: self.config.clone(),
            callbacks: self.callbacks.clone(),
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for FakeListLLM {
    async fn generate(&self, _prompts: &[String], _stop: Option<&[&str]>) -> Result<LLMResult> {
        let text = self.get_response();
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
        let text = self.get_response();
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
impl ChatModel for FakeListLLM {
    async fn predict_messages(
        &self,
        _messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        _stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let text = self.get_response();
        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        _messages: &[BaseMessage],
        _stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let text = self.get_response();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_list_llm_generate() {
        let llm = FakeListLLM::new(vec!["Hello world".to_string()]);
        let result = llm.generate(&["prompt".to_string()], None).await.unwrap();
        assert_eq!(result.generations[0][0].text, "Hello world");
    }

    #[tokio::test]
    async fn test_fake_list_llm_generate_empty_responses() {
        let llm = FakeListLLM::new(vec![]);
        let result = llm.generate(&["prompt".to_string()], None).await.unwrap();
        assert_eq!(result.generations[0][0].text, "");
    }

    #[tokio::test]
    async fn test_fake_list_llm_stream() {
        let llm = FakeListLLM::new(vec!["ab".to_string()]);
        let mut stream = llm.stream(&["prompt".to_string()], None).await.unwrap();
        use futures::StreamExt;
        let mut chunks = Vec::new();
        while let Some(chunk) = stream.next().await {
            chunks.push(chunk.unwrap().text);
        }
        assert_eq!(chunks.join(""), "ab");
    }

    #[tokio::test]
    async fn test_fake_list_llm_predict_messages() {
        let llm = FakeListLLM::new(vec!["response".to_string()]);
        let msg = llm.predict_messages(&[], None, None).await.unwrap();
        assert_eq!(msg.content, "response");
    }

    #[tokio::test]
    async fn test_fake_list_llm_stream_messages() {
        let llm = FakeListLLM::new(vec!["hi".to_string()]);
        let mut stream = llm.stream_messages(&[], None).await.unwrap();
        use futures::StreamExt;
        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk.content, "h");
    }

    #[tokio::test]
    async fn test_fake_list_llm_bind_functions() {
        let llm = FakeListLLM::new(vec!["test".to_string()]);
        let bound = llm.bind_functions(vec![FunctionDefinition {
            name: "my_func".into(),
            description: "A test function".into(),
            parameters: serde_json::json!({"type": "object"}),
        }]);
        let msg = bound.predict_messages(&[], None, None).await.unwrap();
        assert_eq!(msg.content, "test");
    }

    #[tokio::test]
    async fn test_fake_list_llm_with_config() {
        let llm = FakeListLLM::new(vec!["cfg".to_string()]);
        let config = GenerationConfig {
            temperature: Some(0.8),
            ..Default::default()
        };
        let configured = llm.with_config(config);
        let result = configured.generate(&["test".to_string()], None).await.unwrap();
        assert_eq!(result.generations[0][0].text, "cfg");
    }

    #[tokio::test]
    async fn test_fake_list_llm_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FakeListLLM>();
        assert_sync::<FakeListLLM>();
    }
}

pub struct FakeStreamingListLLM {
    responses: Vec<String>,
    config: GenerationConfig,
    callbacks: CallbackManager,
    bound_functions: Vec<FunctionDefinition>,
    bound_tools: Vec<ToolDefinition>,
}

impl FakeStreamingListLLM {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            config: GenerationConfig::default(),
            callbacks: CallbackManager::new(),
            bound_functions: Vec::new(),
            bound_tools: Vec::new(),
        }
    }

    fn get_response(&self) -> String {
        self.responses
            .first()
            .cloned()
            .unwrap_or_default()
    }
}

impl Clone for FakeStreamingListLLM {
    fn clone(&self) -> Self {
        Self {
            responses: self.responses.clone(),
            config: self.config.clone(),
            callbacks: self.callbacks.clone(),
            bound_functions: self.bound_functions.clone(),
            bound_tools: self.bound_tools.clone(),
        }
    }
}

#[async_trait]
impl BaseLLM for FakeStreamingListLLM {
    async fn generate(&self, _prompts: &[String], _stop: Option<&[&str]>) -> Result<LLMResult> {
        let text = self.get_response();
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
        let text = self.get_response();
        let simulated_tokens = text
            .split_whitespace()
            .map(|w| format!("{} ", w));
        let chunks: Vec<Result<GenerationChunk>> = simulated_tokens
            .map(|t| Ok(GenerationChunk::new(t)))
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
impl ChatModel for FakeStreamingListLLM {
    async fn predict_messages(
        &self,
        _messages: &[BaseMessage],
        _functions: Option<&[FunctionDefinition]>,
        _stop: Option<&[&str]>,
    ) -> Result<BaseMessage> {
        let text = self.get_response();
        Ok(BaseMessage::new(text, MessageType::AI))
    }

    async fn stream_messages(
        &self,
        _messages: &[BaseMessage],
        _stop: Option<&[&str]>,
    ) -> Result<BoxStream<'static, Result<MessageChunk>>> {
        let text = self.get_response();
        let simulated_tokens = text
            .split_whitespace()
            .map(|w| format!("{} ", w));
        let chunks: Vec<Result<MessageChunk>> = simulated_tokens
            .map(|t| Ok(MessageChunk::new(t)))
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

#[cfg(test)]
mod tests_streaming {
    use super::*;

    #[tokio::test]
    async fn test_fake_streaming_llm_generate() {
        let llm = FakeStreamingListLLM::new(vec!["streaming response".to_string()]);
        let result = llm.generate(&["prompt".to_string()], None).await.unwrap();
        assert_eq!(result.generations[0][0].text, "streaming response");
    }

    #[tokio::test]
    async fn test_fake_streaming_llm_stream() {
        let llm = FakeStreamingListLLM::new(vec!["hello world".to_string()]);
        let mut stream = llm.stream(&["prompt".to_string()], None).await.unwrap();
        use futures::StreamExt;
        let mut tokens = Vec::new();
        while let Some(chunk) = stream.next().await {
            tokens.push(chunk.unwrap().text);
        }
        assert!(!tokens.is_empty());
    }

    #[tokio::test]
    async fn test_fake_streaming_llm_predict_messages() {
        let llm = FakeStreamingListLLM::new(vec!["chat response".to_string()]);
        let msg = llm.predict_messages(&[], None, None).await.unwrap();
        assert_eq!(msg.content, "chat response");
    }

    #[tokio::test]
    async fn test_fake_streaming_llm_stream_messages() {
        let llm = FakeStreamingListLLM::new(vec!["hello world".to_string()]);
        let mut stream = llm.stream_messages(&[], None).await.unwrap();
        use futures::StreamExt;
        let first = stream.next().await.unwrap().unwrap();
        assert!(!first.content.is_empty());
    }

    #[tokio::test]
    async fn test_fake_streaming_llm_empty_responses() {
        let llm = FakeStreamingListLLM::new(vec![]);
        let result = llm.generate(&["prompt".to_string()], None).await.unwrap();
        assert_eq!(result.generations[0][0].text, "");
    }

    #[tokio::test]
    async fn test_fake_streaming_llm_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FakeStreamingListLLM>();
        assert_sync::<FakeStreamingListLLM>();
    }
}
