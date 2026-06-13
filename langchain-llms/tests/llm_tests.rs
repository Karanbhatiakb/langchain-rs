use futures::StreamExt;
use langchain_core::callbacks::CallbackManager;
use langchain_core::messages::{BaseMessage, MessageType};
use langchain_llms::fake::{FakeListLLM, FakeStreamingListLLM};
use langchain_llms::traits::{BaseLLM, ChatModel};
use langchain_llms::types::{Generation, GenerationChunk, GenerationConfig, LLMResult, MessageChunk};

#[cfg(feature = "xai")]
use langchain_llms::xai::{XaiLLM, ChatXai};
#[cfg(feature = "openrouter")]
use langchain_llms::openrouter::{OpenRouterLLM, ChatOpenRouter};
#[cfg(feature = "ai21")]
use langchain_llms::ai21::{Ai21LLM, ChatAi21};
#[cfg(feature = "cerebras")]
use langchain_llms::cerebras::{CerebrasLLM, ChatCerebras};
#[cfg(feature = "nvidia")]
use langchain_llms::nvidia::{NvidiaLLM, ChatNvidia};
#[cfg(feature = "sambanova")]
use langchain_llms::sambanova::{SambaNovaLLM, ChatSambaNova};
#[cfg(feature = "databricks")]
use langchain_llms::databricks::{DatabricksLLM, ChatDatabricks};
#[cfg(feature = "litellm")]
use langchain_llms::litellm::{LiteLLMLLM, ChatLiteLLM};
#[cfg(feature = "localai")]
use langchain_llms::localai::{LocalAiLLM, ChatLocalAi};

#[tokio::test]
async fn test_fake_list_llm_generate() {
    let llm = FakeListLLM::new(vec!["Hello world".to_string()]);
    let result = llm.generate(&["test prompt".to_string()], None).await.unwrap();
    assert_eq!(result.generations.len(), 1);
    assert_eq!(result.generations[0].len(), 1);
    assert_eq!(result.generations[0][0].text, "Hello world");
}

#[tokio::test]
async fn test_fake_list_llm_generate_returns_message() {
    let llm = FakeListLLM::new(vec!["AI response".to_string()]);
    let result = llm.generate(&["prompt".to_string()], None).await.unwrap();
    let gen = &result.generations[0][0];
    assert!(gen.message.is_some());
    let msg = gen.message.as_ref().unwrap();
    assert_eq!(msg.content, "AI response");
    matches!(msg.message_type, MessageType::AI);
}

#[tokio::test]
async fn test_fake_list_llm_generate_with_stop() {
    let llm = FakeListLLM::new(vec!["Hello".to_string()]);
    let stop = vec!["llo"];
    let result = llm.generate(&["prompt".to_string()], Some(&stop)).await.unwrap();
    assert_eq!(result.generations[0][0].text, "Hello");
}

#[tokio::test]
async fn test_fake_list_llm_empty_responses() {
    let llm = FakeListLLM::new(vec![]);
    let result = llm.generate(&["prompt".to_string()], None).await.unwrap();
    assert_eq!(result.generations[0][0].text, "");
}

#[tokio::test]
async fn test_fake_list_llm_stream() {
    let llm = FakeListLLM::new(vec!["Hi".to_string()]);
    let mut stream = llm.stream(&["prompt".to_string()], None).await.unwrap();
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        chunks.push(chunk.unwrap().text);
    }
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0], "H");
    assert_eq!(chunks[1], "i");
}

#[tokio::test]
async fn test_fake_list_llm_predict_messages() {
    let llm = FakeListLLM::new(vec!["AI reply".to_string()]);
    let messages = vec![BaseMessage::new("Hello", MessageType::Human)];
    let response = llm.predict_messages(&messages, None, None).await.unwrap();
    assert_eq!(response.content, "AI reply");
    matches!(response.message_type, MessageType::AI);
}

#[tokio::test]
async fn test_fake_list_llm_stream_messages() {
    let llm = FakeListLLM::new(vec!["Hi".to_string()]);
    let messages = vec![BaseMessage::new("Hello", MessageType::Human)];
    let mut stream = llm.stream_messages(&messages, None).await.unwrap();
    let mut content = String::new();
    while let Some(chunk) = stream.next().await {
        content.push_str(&chunk.unwrap().content);
    }
    assert_eq!(content, "Hi");
}

#[tokio::test]
async fn test_fake_streaming_list_llm_generate() {
    let llm = FakeStreamingListLLM::new(vec!["Hello world".to_string()]);
    let result = llm.generate(&["prompt".to_string()], None).await.unwrap();
    assert_eq!(result.generations[0][0].text, "Hello world");
}

#[tokio::test]
async fn test_fake_streaming_list_llm_stream() {
    let llm = FakeStreamingListLLM::new(vec!["Hello world test".to_string()]);
    let mut stream = llm.stream(&["prompt".to_string()], None).await.unwrap();
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        chunks.push(chunk.unwrap().text);
    }
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], "Hello ");
    assert_eq!(chunks[1], "world ");
    assert_eq!(chunks[2], "test ");
}

#[tokio::test]
async fn test_fake_streaming_list_llm_predict_messages() {
    let llm = FakeStreamingListLLM::new(vec!["response".to_string()]);
    let messages = vec![BaseMessage::new("question", MessageType::Human)];
    let response = llm.predict_messages(&messages, None, None).await.unwrap();
    assert_eq!(response.content, "response");
}

#[tokio::test]
async fn test_fake_streaming_list_llm_stream_messages() {
    let llm = FakeStreamingListLLM::new(vec!["Hello world".to_string()]);
    let messages = vec![BaseMessage::new("Hello", MessageType::Human)];
    let mut stream = llm.stream_messages(&messages, None).await.unwrap();
    let mut parts = Vec::new();
    while let Some(chunk) = stream.next().await {
        parts.push(chunk.unwrap().content);
    }
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "Hello ");
    assert_eq!(parts[1], "world ");
}

#[tokio::test]
async fn test_with_config() {
    let llm = FakeListLLM::new(vec!["test".to_string()]);
    let config = GenerationConfig {
        temperature: Some(0.5),
        max_tokens: Some(100),
        ..GenerationConfig::default()
    };
    let _configured = llm.with_config(config);
}

#[tokio::test]
async fn test_with_callbacks() {
    let llm = FakeListLLM::new(vec!["test".to_string()]);
    let callbacks = CallbackManager::new();
    let _with_cb = llm.with_callbacks(callbacks);
}

#[test]
fn test_generation_chunk_new() {
    let chunk = GenerationChunk::new("hello");
    assert_eq!(chunk.text, "hello");
    assert!(chunk.generation_info.is_none());
}

#[test]
fn test_message_chunk_new() {
    let chunk = MessageChunk::new("content");
    assert_eq!(chunk.content, "content");
    assert!(chunk.additional_kwargs.is_empty());
}

#[test]
fn test_generation_config_default() {
    let config = GenerationConfig::default();
    assert!(config.temperature.is_none());
    assert!(config.max_tokens.is_none());
    assert!(config.top_p.is_none());
    assert!(config.frequency_penalty.is_none());
    assert!(config.presence_penalty.is_none());
    assert!(config.stop_sequences.is_none());
    assert!(config.model.is_none());
    assert!(config.seed.is_none());
}

#[test]
fn test_llm_result_serialization() {
    let result = LLMResult {
        generations: vec![vec![Generation {
            text: "hello".to_string(),
            message: None,
            generation_info: None,
        }]],
        llm_output: None,
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: LLMResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.generations[0][0].text, "hello");
}

#[cfg(feature = "xai")]
mod xai_tests {
    use super::*;
    #[test]
    fn test_xai_llm_creation() {
        let _llm = XaiLLM::new("grok-2", "test-key");
    }
    #[test]
    fn test_xai_llm_with_base_url() {
        let _llm = XaiLLM::new("grok-2", "test-key")
            .with_base_url("https://custom.api.x.ai");
    }
    #[test]
    fn test_xai_llm_with_timeout() {
        let _llm = XaiLLM::new("grok-2", "test-key")
            .with_timeout(std::time::Duration::from_secs(30));
    }
    #[test]
    fn test_chat_xai_creation() {
        let _chat = ChatXai::new("grok-2", "test-key");
    }
    #[test]
    fn test_chat_xai_with_base_url() {
        let _chat = ChatXai::new("grok-2", "test-key")
            .with_base_url("https://custom.api.x.ai");
    }
}

#[cfg(feature = "openrouter")]
mod openrouter_tests {
    use super::*;
    #[test]
    fn test_openrouter_llm_creation() {
        let _llm = OpenRouterLLM::new("meta-llama/llama-3", "test-key");
    }
    #[test]
    fn test_openrouter_llm_with_base_url() {
        let _llm = OpenRouterLLM::new("meta-llama/llama-3", "test-key")
            .with_base_url("https://openrouter.ai/api/v1");
    }
    #[test]
    fn test_openrouter_llm_with_site_url() {
        let _llm = OpenRouterLLM::new("meta-llama/llama-3", "test-key")
            .with_site_url("https://example.com");
    }
    #[test]
    fn test_openrouter_llm_with_site_name() {
        let _llm = OpenRouterLLM::new("meta-llama/llama-3", "test-key")
            .with_site_name("MyApp");
    }
    #[test]
    fn test_chat_openrouter_creation() {
        let _chat = ChatOpenRouter::new("meta-llama/llama-3", "test-key");
    }
}

#[cfg(feature = "ai21")]
mod ai21_tests {
    use super::*;
    #[test]
    fn test_ai21_llm_creation() {
        let _llm = Ai21LLM::new("jamba-1.5", "test-key");
    }
    #[test]
    fn test_ai21_llm_with_base_url() {
        let _llm = Ai21LLM::new("jamba-1.5", "test-key")
            .with_base_url("https://api.ai21.com/studio/v1");
    }
    #[test]
    fn test_chat_ai21_creation() {
        let _chat = ChatAi21::new("jamba-1.5", "test-key");
    }
}

#[cfg(feature = "cerebras")]
mod cerebras_tests {
    use super::*;
    #[test]
    fn test_cerebras_llm_creation() {
        let _llm = CerebrasLLM::new("llama3.1-8b", "test-key");
    }
    #[test]
    fn test_cerebras_llm_with_base_url() {
        let _llm = CerebrasLLM::new("llama3.1-8b", "test-key")
            .with_base_url("https://api.cerebras.ai/v1");
    }
    #[test]
    fn test_chat_cerebras_creation() {
        let _chat = ChatCerebras::new("llama3.1-8b", "test-key");
    }
}

#[cfg(feature = "nvidia")]
mod nvidia_tests {
    use super::*;
    #[test]
    fn test_nvidia_llm_creation() {
        let _llm = NvidiaLLM::new("nemotron", "test-key");
    }
    #[test]
    fn test_nvidia_llm_with_base_url() {
        let _llm = NvidiaLLM::new("nemotron", "test-key")
            .with_base_url("https://integrate.api.nvidia.com/v1");
    }
    #[test]
    fn test_chat_nvidia_creation() {
        let _chat = ChatNvidia::new("nemotron", "test-key");
    }
}

#[cfg(feature = "sambanova")]
mod sambanova_tests {
    use super::*;
    #[test]
    fn test_sambanova_llm_creation() {
        let _llm = SambaNovaLLM::new("samba-1", "test-key");
    }
    #[test]
    fn test_sambanova_llm_with_base_url() {
        let _llm = SambaNovaLLM::new("samba-1", "test-key")
            .with_base_url("https://api.sambanova.ai/v1");
    }
    #[test]
    fn test_chat_sambanova_creation() {
        let _chat = ChatSambaNova::new("samba-1", "test-key");
    }
}

#[cfg(feature = "databricks")]
mod databricks_tests {
    use super::*;
    #[test]
    fn test_databricks_llm_creation() {
        let _llm = DatabricksLLM::new("dbrx", "test-key");
    }
    #[test]
    fn test_databricks_llm_with_base_url() {
        let _llm = DatabricksLLM::new("dbrx", "test-key")
            .with_base_url("https://api.databricks.com/serving-endpoints");
    }
    #[test]
    fn test_chat_databricks_creation() {
        let _chat = ChatDatabricks::new("dbrx", "test-key");
    }
}

#[cfg(feature = "litellm")]
mod litellm_tests {
    use super::*;
    #[test]
    fn test_litellm_llm_creation() {
        let _llm = LiteLLMLLM::new("gpt-4", "test-key");
    }
    #[test]
    fn test_litellm_llm_with_base_url() {
        let _llm = LiteLLMLLM::new("gpt-4", "test-key")
            .with_base_url("http://localhost:4000");
    }
    #[test]
    fn test_chat_litellm_creation() {
        let _chat = ChatLiteLLM::new("gpt-4", "test-key");
    }
}

#[cfg(feature = "localai")]
mod localai_tests {
    use super::*;
    #[test]
    fn test_localai_llm_creation() {
        let _llm = LocalAiLLM::new("llama3");
    }
    #[test]
    fn test_localai_llm_with_base_url() {
        let _llm = LocalAiLLM::new("llama3")
            .with_base_url("http://localhost:8080");
    }
    #[test]
    fn test_localai_llm_with_api_key() {
        let _llm = LocalAiLLM::new("llama3")
            .with_api_key("test-key");
    }
    #[test]
    fn test_chat_localai_creation() {
        let _chat = ChatLocalAi::new("llama3");
    }
}
