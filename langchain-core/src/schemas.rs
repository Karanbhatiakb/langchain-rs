//! Schema types for LLM generations, chat results, and streaming chunks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single text generation from an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
    /// The generated text.
    pub text: String,
    /// Optional extra information from the provider (e.g., logprobs).
    pub generation_info: Option<HashMap<String, serde_json::Value>>,
}

impl Generation {
    /// Creates a new `Generation` with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            generation_info: None,
        }
    }
}

/// A chunk of a generation, used in streaming contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationChunk {
    /// The text fragment.
    pub text: String,
    /// Optional extra information attached to this chunk.
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

/// The complete output of an LLM call, containing one or more generations per
/// prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResult {
    /// Outer vec: one entry per input prompt. Inner vec: candidate generations.
    pub generations: Vec<Vec<Generation>>,
    /// Optional metadata from the LLM provider (e.g., token usage).
    pub llm_output: Option<HashMap<String, serde_json::Value>>,
}

impl LLMResult {
    /// Creates a new `LLMResult` from the provided generations.
    pub fn new(generations: Vec<Vec<Generation>>) -> Self {
        Self {
            generations,
            llm_output: None,
        }
    }
}

/// A single generation from a chat model, including the full message object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGeneration {
    /// The text of the generated message.
    pub text: String,
    /// The structured message object.
    pub message: crate::messages::BaseMessage,
    /// Optional extra information from the provider.
    pub generation_info: Option<HashMap<String, serde_json::Value>>,
}

impl ChatGeneration {
    /// Creates a new `ChatGeneration` from a message.
    pub fn new(message: impl Into<crate::messages::BaseMessage>) -> Self {
        let base: crate::messages::BaseMessage = message.into();
        Self {
            text: base.content.clone(),
            message: base,
            generation_info: None,
        }
    }
}

/// The result of a chat model invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResult {
    /// The candidate chat generations.
    pub generations: Vec<ChatGeneration>,
    /// Optional provider-level metadata.
    pub llm_output: Option<HashMap<String, serde_json::Value>>,
}

/// A streaming chunk for chat messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageChunk {
    /// The partial text content.
    pub content: String,
    /// The message role / origin.
    pub message_type: crate::messages::MessageType,
    /// An optional sender name.
    pub name: Option<String>,
    /// Extra key-value data for this chunk.
    pub additional_kwargs: HashMap<String, serde_json::Value>,
}

impl MessageChunk {
    /// Creates a new `MessageChunk` with the given content and type.
    pub fn new(content: impl Into<String>, message_type: crate::messages::MessageType) -> Self {
        Self {
            content: content.into(),
            message_type,
            name: None,
            additional_kwargs: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::MessageType;

    #[test]
    fn test_generation_new() {
        let g = Generation::new("hello");
        assert_eq!(g.text, "hello");
        assert!(g.generation_info.is_none());
    }

    #[test]
    fn test_generation_chunk_new() {
        let chunk = GenerationChunk::new("hello");
        assert_eq!(chunk.text, "hello");
        assert!(chunk.generation_info.is_none());
    }

    #[test]
    fn test_llm_result_new() {
        let g = Generation::new("response");
        let result = LLMResult::new(vec![vec![g]]);
        assert_eq!(result.generations.len(), 1);
        assert!(result.llm_output.is_none());
    }

    #[test]
    fn test_llm_result_empty() {
        let result = LLMResult::new(vec![]);
        assert!(result.generations.is_empty());
    }

    #[test]
    fn test_chat_generation_new() {
        let msg = crate::messages::BaseMessage::new("hi", MessageType::AI);
        let cg = ChatGeneration::new(msg);
        assert_eq!(cg.text, "hi");
        assert!(cg.generation_info.is_none());
    }

    #[test]
    fn test_chat_result_new() {
        let msg = crate::messages::BaseMessage::new("hi", MessageType::AI);
        let cg = ChatGeneration::new(msg);
        let result = ChatResult {
            generations: vec![cg],
            llm_output: None,
        };
        assert_eq!(result.generations.len(), 1);
    }

    #[test]
    fn test_message_chunk_new() {
        let chunk = MessageChunk::new("hello", MessageType::Human);
        assert_eq!(chunk.content, "hello");
        assert!(matches!(chunk.message_type, MessageType::Human));
        assert!(chunk.name.is_none());
    }

    #[test]
    fn test_message_chunk_with_name() {
        let chunk = MessageChunk {
            content: "hello".into(),
            message_type: MessageType::AI,
            name: Some("assistant".into()),
            additional_kwargs: HashMap::new(),
        };
        assert_eq!(chunk.name.unwrap(), "assistant");
    }

    #[test]
    fn test_generation_with_info() {
        let mut info = HashMap::new();
        info.insert("finish_reason".into(), serde_json::Value::String("stop".into()));
        let g = Generation {
            text: "ok".into(),
            generation_info: Some(info),
        };
        let fin = g.generation_info.as_ref().unwrap().get("finish_reason").unwrap();
        assert_eq!(fin, "stop");
    }

    #[test]
    fn test_generation_serde() {
        let g = Generation::new("hello");
        let json = serde_json::to_string(&g).unwrap();
        let deserialized: Generation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text, "hello");
    }

    #[test]
    fn test_schemas_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Generation>();
        assert_sync::<Generation>();
        assert_send::<GenerationChunk>();
        assert_sync::<GenerationChunk>();
        assert_send::<LLMResult>();
        assert_sync::<LLMResult>();
        assert_send::<ChatGeneration>();
        assert_sync::<ChatGeneration>();
        assert_send::<MessageChunk>();
        assert_sync::<MessageChunk>();
    }
}
