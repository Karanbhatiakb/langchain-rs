//! Types for LLM generations, chunks, results, and configuration.

use langchain_core::messages::BaseMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::traits::{FunctionDefinition, ToolDefinition};

/// A single generation result from an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
    /// The generated text.
    pub text: String,
    /// An optional structured message (populated for chat models).
    pub message: Option<BaseMessage>,
    /// Optional provider-specific metadata (e.g., logprobs).
    pub generation_info: Option<HashMap<String, serde_json::Value>>,
}

/// The complete output of one or more generation prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResult {
    /// Outer vec: one entry per prompt. Inner vec: candidate generations.
    pub generations: Vec<Vec<Generation>>,
    /// Optional provider-level output metadata.
    pub llm_output: Option<HashMap<String, serde_json::Value>>,
}

/// A streaming chunk of a generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationChunk {
    /// The partial text content.
    pub text: String,
    /// Optional provider-specific chunk metadata.
    pub generation_info: Option<HashMap<String, serde_json::Value>>,
}

impl GenerationChunk {
    /// Creates a new `GenerationChunk` with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            generation_info: None,
        }
    }
}

/// A streaming chunk of a chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageChunk {
    /// The partial text content.
    pub content: String,
    /// Extra data attached to this chunk (e.g., function call deltas).
    pub additional_kwargs: HashMap<String, serde_json::Value>,
}

impl MessageChunk {
    /// Creates a new `MessageChunk` with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            additional_kwargs: HashMap::new(),
        }
    }
}

/// Configuration for LLM generation requests.
///
/// All fields are optional; providers use sensible defaults when a field is
/// `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Sampling temperature (0.0–2.0).
    pub temperature: Option<f64>,
    /// Maximum number of tokens to generate.
    pub max_tokens: Option<u32>,
    /// Nucleus sampling parameter (0.0–1.0).
    pub top_p: Option<f64>,
    /// Frequency penalty (-2.0–2.0).
    pub frequency_penalty: Option<f64>,
    /// Presence penalty (-2.0–2.0).
    pub presence_penalty: Option<f64>,
    /// Sequences where generation should stop.
    pub stop_sequences: Option<Vec<String>>,
    /// Model identifier override.
    pub model: Option<String>,
    /// Function definitions for function calling.
    pub functions: Option<Vec<FunctionDefinition>>,
    /// Tool definitions for tool calling.
    pub tools: Option<Vec<ToolDefinition>>,
    /// Random seed for deterministic generation.
    pub seed: Option<u64>,
    /// Top-k sampling parameter.
    pub top_k: Option<u32>,
    /// Number of completions to generate.
    pub n: Option<u32>,
    /// Logit bias map (token ID → bias value).
    pub logit_bias: Option<HashMap<String, f64>>,
    /// User identifier for tracking.
    pub user: Option<String>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: None,
            model: None,
            functions: None,
            tools: None,
            seed: None,
            top_k: None,
            n: None,
            logit_bias: None,
            user: None,
        }
    }
}

/// Trait for LLM provider identification.
pub trait LLMProvider {
    /// Returns the provider name string.
    fn provider_name(&self) -> &'static str;
}

/// Trait for accessing provider configuration.
pub trait ProviderConfig {
    /// Returns a reference to the generation config.
    fn config(&self) -> &GenerationConfig;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_generation_new() {
        let g = Generation {
            text: "hello".into(),
            message: None,
            generation_info: None,
        };
        assert_eq!(g.text, "hello");
        assert!(g.message.is_none());
        assert!(g.generation_info.is_none());
    }

    #[test]
    fn test_generation_with_message() {
        let msg = BaseMessage::new("response", langchain_core::messages::MessageType::AI);
        let g = Generation {
            text: "response".into(),
            message: Some(msg.clone()),
            generation_info: None,
        };
        assert_eq!(g.text, "response");
        assert_eq!(g.message.as_ref().unwrap().content, "response");
    }

    #[test]
    fn test_generation_with_info() {
        let mut info = HashMap::new();
        info.insert("finish_reason".into(), serde_json::Value::String("stop".into()));
        let g = Generation {
            text: "ok".into(),
            message: None,
            generation_info: Some(info),
        };
        let fin = g.generation_info.as_ref().unwrap().get("finish_reason").unwrap();
        assert_eq!(fin, "stop");
    }

    #[test]
    fn test_llm_result_new() {
        let gen = Generation {
            text: "a".into(),
            message: None,
            generation_info: None,
        };
        let result = LLMResult {
            generations: vec![vec![gen]],
            llm_output: None,
        };
        assert_eq!(result.generations.len(), 1);
        assert_eq!(result.generations[0][0].text, "a");
        assert!(result.llm_output.is_none());
    }

    #[test]
    fn test_llm_result_with_metadata() {
        let gen = Generation {
            text: "b".into(),
            message: None,
            generation_info: None,
        };
        let mut meta = HashMap::new();
        meta.insert("token_usage".into(), serde_json::json!({"total": 10}));
        let result = LLMResult {
            generations: vec![vec![gen]],
            llm_output: Some(meta),
        };
        assert!(result.llm_output.is_some());
    }

    #[test]
    fn test_generation_chunk_new() {
        let chunk = GenerationChunk::new("hello");
        assert_eq!(chunk.text, "hello");
        assert!(chunk.generation_info.is_none());
    }

    #[test]
    fn test_message_chunk_new() {
        let chunk = MessageChunk::new("hello");
        assert_eq!(chunk.content, "hello");
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
        assert!(config.top_k.is_none());
        assert!(config.n.is_none());
        assert!(config.user.is_none());
    }

    #[test]
    fn test_generation_config_partial() {
        let config = GenerationConfig {
            temperature: Some(0.7),
            max_tokens: Some(100),
            model: Some("gpt-4".into()),
            ..Default::default()
        };
        assert!((config.temperature.unwrap() - 0.7).abs() < f64::EPSILON);
        assert_eq!(config.max_tokens.unwrap(), 100);
        assert_eq!(config.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_send_sync_traits() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<Generation>();
        assert_send_sync::<LLMResult>();
        assert_send_sync::<GenerationChunk>();
        assert_send_sync::<MessageChunk>();
        assert_send_sync::<GenerationConfig>();
    }

    #[test]
    fn test_llm_result_multiple_generations() {
        let gens = vec![
            vec![Generation { text: "a".into(), message: None, generation_info: None }],
            vec![Generation { text: "b".into(), message: None, generation_info: None }],
            vec![Generation { text: "c".into(), message: None, generation_info: None }],
        ];
        let result = LLMResult { generations: gens, llm_output: None };
        assert_eq!(result.generations.len(), 3);
    }

    #[test]
    fn test_generation_chunk_with_info() {
        let mut info = HashMap::new();
        info.insert("token".into(), serde_json::json!(42));
        let chunk = GenerationChunk {
            text: "hello".into(),
            generation_info: Some(info),
        };
        assert!(chunk.generation_info.is_some());
    }

    #[test]
    fn test_generation_config_full() {
        let config = GenerationConfig {
            temperature: Some(0.5),
            max_tokens: Some(200),
            top_p: Some(0.9),
            frequency_penalty: Some(0.1),
            presence_penalty: Some(0.2),
            stop_sequences: Some(vec!["END".into()]),
            model: Some("gpt-4".into()),
            seed: Some(42),
            top_k: Some(50),
            n: Some(2),
            user: Some("test_user".into()),
            ..Default::default()
        };
        assert_eq!(config.model.as_deref(), Some("gpt-4"));
        assert_eq!(config.seed, Some(42));
        assert_eq!(config.n, Some(2));
    }

    #[test]
    fn test_llm_provider_trait_object() {
        struct DummyProvider;
        impl LLMProvider for DummyProvider {
            fn provider_name(&self) -> &'static str { "dummy" }
        }
        let p: &dyn LLMProvider = &DummyProvider;
        assert_eq!(p.provider_name(), "dummy");
    }

    #[test]
    fn test_provider_config_trait() {
        struct DummyProvider { config: GenerationConfig }
        impl ProviderConfig for DummyProvider {
            fn config(&self) -> &GenerationConfig { &self.config }
        }
        let p = DummyProvider { config: GenerationConfig { temperature: Some(0.5), ..Default::default() } };
        assert!((p.config().temperature.unwrap() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_message_chunk_with_kwargs() {
        let mut chunk = MessageChunk::new("hello");
        chunk.additional_kwargs.insert("role".into(), serde_json::json!("assistant"));
        assert_eq!(chunk.additional_kwargs.len(), 1);
    }

    #[test]
    fn test_llm_result_with_empty_generations() {
        let result = LLMResult { generations: vec![], llm_output: None };
        assert!(result.generations.is_empty());
    }
}
