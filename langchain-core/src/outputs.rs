//! LLM and chat model output schemas.
//!
//! Defines the types used to represent generation results from language
//! models: [`Generation`] and [`ChatGeneration`] for individual outputs,
//! [`LLMResult`] and [`ChatResult`] for batched results, and streaming
//! chunk types [`GenerationChunk`] and [`ChatGenerationChunk`] with
//! merge semantics.

use crate::messages::{BaseMessage, MessageType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single text generation output from an LLM.
///
/// This mirrors the Python `Generation` class — it carries the generated
/// text and optional provider-specific metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
    /// The generated text.
    pub text: String,
    /// Optional extra information from the provider (e.g., finish reason,
    /// logprobs).
    pub generation_info: Option<serde_json::Value>,
}

impl Generation {
    /// Creates a new `Generation` with the given text and no metadata.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            generation_info: None,
        }
    }

    /// Sets the `generation_info` field (builder pattern).
    pub fn with_generation_info(mut self, info: serde_json::Value) -> Self {
        self.generation_info = Some(info);
        self
    }
}

/// A single chat generation output from a chat model.
///
/// Extends the text-based [`Generation`] concept with the structured
/// [`BaseMessage`] produced by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGeneration {
    /// The structured message output by the chat model.
    pub message: BaseMessage,
    /// Optional extra information from the provider.
    pub generation_info: Option<serde_json::Value>,
    /// The text contents of the output message.
    pub text: String,
}

impl ChatGeneration {
    /// Creates a new `ChatGeneration` from a message.
    ///
    /// The `text` field is automatically set from the message content.
    pub fn new(message: BaseMessage) -> Self {
        let text = message.content.clone();
        Self {
            message,
            generation_info: None,
            text,
        }
    }

    /// Sets the `generation_info` field (builder pattern).
    pub fn with_generation_info(mut self, info: serde_json::Value) -> Self {
        self.generation_info = Some(info);
        self
    }
}

/// Metadata for a single execution of a chain or model.
///
/// Carries a unique [`Uuid`] identifying the run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunInfo {
    /// A unique identifier for the model or chain run.
    pub run_id: Uuid,
}

impl RunInfo {
    /// Creates a new `RunInfo` with a random UUID.
    pub fn new() -> Self {
        Self {
            run_id: Uuid::new_v4(),
        }
    }

    /// Creates a `RunInfo` with the given run ID.
    pub fn with_run_id(run_id: Uuid) -> Self {
        Self { run_id }
    }
}

impl Default for RunInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// The complete output of an LLM call, containing one or more generations
/// per prompt.
///
/// The outer `Vec` has one entry per input prompt; the inner `Vec` holds
/// candidate generations for that prompt. Optional provider-level metadata
/// and per-prompt run info are also carried.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResult {
    /// Generated outputs, indexed by prompt then by candidate.
    pub generations: Vec<Vec<Generation>>,
    /// Optional provider-level metadata (e.g., token usage).
    pub llm_output: Option<serde_json::Value>,
    /// Optional per-prompt run metadata.
    pub run: Option<Vec<RunInfo>>,
}

impl LLMResult {
    /// Creates a new `LLMResult` from the provided generations.
    pub fn new(generations: Vec<Vec<Generation>>) -> Self {
        Self {
            generations,
            llm_output: None,
            run: None,
        }
    }

    /// Sets the `llm_output` field (builder pattern).
    pub fn with_llm_output(mut self, output: serde_json::Value) -> Self {
        self.llm_output = Some(output);
        self
    }

    /// Sets the `run` field (builder pattern).
    pub fn with_run(mut self, run: Vec<RunInfo>) -> Self {
        self.run = Some(run);
        self
    }

    /// Flattens the generations into one `LLMResult` per prompt.
    ///
    /// Each returned `LLMResult` contains only the generations for a single
    /// input prompt. Token usage metadata is kept only for the first result
    /// to avoid double-counting.
    pub fn flatten(&self) -> Vec<LLMResult> {
        self.generations
            .iter()
            .enumerate()
            .map(|(i, gen_list)| {
                let llm_output = if i == 0 {
                    self.llm_output.clone()
                } else {
                    self.llm_output.as_ref().map(|_| {
                        serde_json::json!({"token_usage": {}})
                    })
                };
                LLMResult {
                    generations: vec![gen_list.clone()],
                    llm_output,
                    run: None,
                }
            })
            .collect()
    }
}

/// The result of a chat model invocation.
///
/// Contains candidate [`ChatGeneration`]s and optional provider metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResult {
    /// Candidate chat generations.
    pub generations: Vec<Vec<ChatGeneration>>,
    /// Optional provider-level metadata.
    pub llm_output: Option<serde_json::Value>,
}

impl ChatResult {
    /// Creates a new `ChatResult` from the provided generations.
    pub fn new(generations: Vec<Vec<ChatGeneration>>) -> Self {
        Self {
            generations,
            llm_output: None,
        }
    }

    /// Sets the `llm_output` field (builder pattern).
    pub fn with_llm_output(mut self, output: serde_json::Value) -> Self {
        self.llm_output = Some(output);
        self
    }
}

/// A streaming chunk of a text generation.
///
/// Chunks can be merged with the `+` operator to incrementally build up
/// a complete generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationChunk {
    /// The text fragment.
    pub text: String,
    /// Optional extra information attached to this chunk.
    pub generation_info: Option<serde_json::Value>,
}

impl GenerationChunk {
    /// Creates a new `GenerationChunk` with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            generation_info: None,
        }
    }

    /// Sets the `generation_info` field (builder pattern).
    pub fn with_generation_info(mut self, info: serde_json::Value) -> Self {
        self.generation_info = Some(info);
        self
    }

    /// Merges another [`GenerationChunk`] into this one.
    ///
    /// Text is concatenated. `generation_info` objects are merged — keys
    /// from `other` overwrite keys from `self`.
    pub fn merge(&mut self, other: &GenerationChunk) {
        self.text.push_str(&other.text);
        if let Some(other_info) = &other.generation_info {
            match &mut self.generation_info {
                Some(self_info) => {
                    if let (Some(self_map), Some(other_map)) =
                        (self_info.as_object_mut(), other_info.as_object())
                    {
                        for (k, v) in other_map {
                            self_map.insert(k.clone(), v.clone());
                        }
                    }
                }
                None => self.generation_info = Some(other_info.clone()),
            }
        }
    }
}

impl std::ops::Add for GenerationChunk {
    type Output = GenerationChunk;

    fn add(self, rhs: GenerationChunk) -> Self::Output {
        let mut result = self.clone();
        result.merge(&rhs);
        result
    }
}

/// A streaming chunk of a chat generation.
///
/// Carries a partial [`BaseMessage`] and optional generation metadata.
/// Chunks can be merged with the `+` operator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGenerationChunk {
    /// The partial message chunk.
    pub message: BaseMessage,
    /// Optional extra information attached to this chunk.
    pub generation_info: Option<serde_json::Value>,
    /// The text contents of the chunk.
    pub text: String,
}

impl ChatGenerationChunk {
    /// Creates a new `ChatGenerationChunk` from a partial message.
    pub fn new(message: BaseMessage) -> Self {
        let text = message.content.clone();
        Self {
            message,
            generation_info: None,
            text,
        }
    }

    /// Creates a new text-only `ChatGenerationChunk` with the given content
    /// and message type.
    pub fn from_text(text: impl Into<String>, message_type: MessageType) -> Self {
        let content = text.into();
        Self {
            message: BaseMessage::new(&content, message_type),
            generation_info: None,
            text: content,
        }
    }

    /// Sets the `generation_info` field (builder pattern).
    pub fn with_generation_info(mut self, info: serde_json::Value) -> Self {
        self.generation_info = Some(info);
        self
    }

    /// Merges another [`ChatGenerationChunk`] into this one.
    ///
    /// Message content is concatenated. `generation_info` objects are merged
    /// — keys from `other` overwrite keys from `self`.
    pub fn merge(&mut self, other: &ChatGenerationChunk) {
        self.message.content.push_str(&other.message.content);
        self.text.push_str(&other.text);
        if let Some(other_info) = &other.generation_info {
            match &mut self.generation_info {
                Some(self_info) => {
                    if let (Some(self_map), Some(other_map)) =
                        (self_info.as_object_mut(), other_info.as_object())
                    {
                        for (k, v) in other_map {
                            self_map.insert(k.clone(), v.clone());
                        }
                    }
                }
                None => self.generation_info = Some(other_info.clone()),
            }
        }
    }
}

impl std::ops::Add for ChatGenerationChunk {
    type Output = ChatGenerationChunk;

    fn add(self, rhs: ChatGenerationChunk) -> Self::Output {
        let mut result = self.clone();
        result.merge(&rhs);
        result
    }
}

/// Merges a list of [`ChatGenerationChunk`]s into a single chunk.
///
/// Returns `None` if the input list is empty.
pub fn merge_chat_generation_chunks(
    chunks: &[ChatGenerationChunk],
) -> Option<ChatGenerationChunk> {
    if chunks.is_empty() {
        return None;
    }
    let mut result = chunks[0].clone();
    for chunk in &chunks[1..] {
        result.merge(chunk);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_new() {
        let g = Generation::new("hello");
        assert_eq!(g.text, "hello");
        assert!(g.generation_info.is_none());
    }

    #[test]
    fn test_chat_generation_new() {
        let msg = BaseMessage::new("hi", MessageType::AI);
        let cg = ChatGeneration::new(msg);
        assert_eq!(cg.text, "hi");
        assert_eq!(cg.message.content, "hi");
    }

    #[test]
    fn test_run_info_default() {
        let ri = RunInfo::default();
        assert_ne!(ri.run_id, Uuid::nil());
    }

    #[test]
    fn test_llm_result_flatten() {
        let g1 = Generation::new("a");
        let g2 = Generation::new("b");
        let result = LLMResult::new(vec![vec![g1], vec![g2]])
            .with_llm_output(serde_json::json!({"token_usage": {"total": 10}}));
        let flat = result.flatten();
        assert_eq!(flat.len(), 2);
        assert!(flat[0].llm_output.is_some());
    }

    #[test]
    fn test_generation_chunk_add() {
        let a = GenerationChunk::new("hello");
        let b = GenerationChunk::new(" world");
        let combined = a + b;
        assert_eq!(combined.text, "hello world");
    }

    #[test]
    fn test_generation_chunk_merge_with_info() {
        let a = GenerationChunk::new("hello")
            .with_generation_info(serde_json::json!({"finish_reason": null}));
        let b = GenerationChunk::new(" world")
            .with_generation_info(serde_json::json!({"finish_reason": "stop"}));
        let combined = a + b;
        assert_eq!(combined.text, "hello world");
        let info = combined.generation_info.unwrap();
        assert_eq!(info["finish_reason"], "stop");
    }

    #[test]
    fn test_chat_generation_chunk_add() {
        let a = ChatGenerationChunk::from_text("hello", MessageType::AI);
        let b = ChatGenerationChunk::from_text(" world", MessageType::AI);
        let combined = a + b;
        assert_eq!(combined.text, "hello world");
        assert_eq!(combined.message.content, "hello world");
    }

    #[test]
    fn test_merge_chat_generation_chunks_empty() {
        assert!(merge_chat_generation_chunks(&[]).is_none());
    }

    #[test]
    fn test_merge_chat_generation_chunks_single() {
        let chunk = ChatGenerationChunk::from_text("hello", MessageType::AI);
        let merged = merge_chat_generation_chunks(&[chunk.clone()]);
        assert!(merged.is_some());
        assert_eq!(merged.unwrap().text, "hello");
    }

    #[test]
    fn test_generation_with_generation_info() {
        let g = Generation::new("test")
            .with_generation_info(serde_json::json!({"finish_reason": "stop"}));
        assert_eq!(g.text, "test");
        let info = g.generation_info.unwrap();
        assert_eq!(info["finish_reason"], "stop");
    }

    #[test]
    fn test_chat_generation_with_generation_info() {
        let msg = BaseMessage::new("hi", MessageType::AI);
        let cg = ChatGeneration::new(msg)
            .with_generation_info(serde_json::json!({"finish_reason": "stop"}));
        assert_eq!(cg.text, "hi");
        let info = cg.generation_info.unwrap();
        assert_eq!(info["finish_reason"], "stop");
    }

    #[test]
    fn test_llm_result_empty() {
        let result = LLMResult::new(vec![]);
        assert!(result.generations.is_empty());
        assert!(result.llm_output.is_none());
        assert!(result.run.is_none());
    }

    #[test]
    fn test_llm_result_with_run() {
        let g = Generation::new("a");
        let run_info = RunInfo::new();
        let result = LLMResult::new(vec![vec![g]]).with_run(vec![run_info]);
        assert!(result.run.is_some());
    }

    #[test]
    fn test_generation_chunk_no_info_merge() {
        let a = GenerationChunk::new("hello");
        let b = GenerationChunk::new(" world");
        let mut merged = a.clone();
        merged.merge(&b);
        assert_eq!(merged.text, "hello world");
        assert!(merged.generation_info.is_none());
    }

    #[test]
    fn test_generation_chunk_info_merge_asymmetric() {
        let a = GenerationChunk::new("hello")
            .with_generation_info(serde_json::json!({"a": 1}));
        let b = GenerationChunk::new(" world");
        let combined = a + b;
        assert_eq!(combined.text, "hello world");
        assert!(combined.generation_info.is_some());
    }

    #[test]
    fn test_chat_generation_chunk_with_generation_info() {
        let msg = BaseMessage::new("hello", MessageType::AI);
        let chunk = ChatGenerationChunk::new(msg)
            .with_generation_info(serde_json::json!({"token_id": 42}));
        assert_eq!(chunk.text, "hello");
        let info = chunk.generation_info.unwrap();
        assert_eq!(info["token_id"], 42);
    }

    #[test]
    fn test_chat_generation_chunk_merge_with_info() {
        let a = ChatGenerationChunk::from_text("hello", MessageType::AI)
            .with_generation_info(serde_json::json!({"a": 1}));
        let b = ChatGenerationChunk::from_text(" world", MessageType::AI)
            .with_generation_info(serde_json::json!({"b": 2}));
        let combined = a + b;
        assert_eq!(combined.text, "hello world");
        let info = combined.generation_info.unwrap();
        assert_eq!(info["a"], 1);
        assert_eq!(info["b"], 2);
    }

    #[test]
    fn test_chat_result_new() {
        let msg = BaseMessage::new("hi", MessageType::AI);
        let cg = ChatGeneration::new(msg);
        let result = ChatResult::new(vec![vec![cg]]);
        assert_eq!(result.generations.len(), 1);
        assert_eq!(result.generations[0][0].text, "hi");
    }

    #[test]
    fn test_chat_result_with_llm_output() {
        let msg = BaseMessage::new("hi", MessageType::AI);
        let cg = ChatGeneration::new(msg);
        let result = ChatResult::new(vec![vec![cg]])
            .with_llm_output(serde_json::json!({"model": "gpt"}));
        let output = result.llm_output.unwrap();
        assert_eq!(output["model"], "gpt");
    }

    #[test]
    fn test_run_info_with_run_id() {
        let uuid = uuid::Uuid::new_v4();
        let ri = RunInfo::with_run_id(uuid);
        assert_eq!(ri.run_id, uuid);
    }

    #[test]
    fn test_chat_generation_chunk_from_text() {
        let chunk = ChatGenerationChunk::from_text("hello", MessageType::Human);
        assert_eq!(chunk.text, "hello");
        assert_eq!(chunk.message.content, "hello");
        assert!(matches!(chunk.message.message_type, MessageType::Human));
    }

    #[test]
    fn test_outputs_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Generation>();
        assert_sync::<Generation>();
        assert_send::<ChatGeneration>();
        assert_sync::<ChatGeneration>();
        assert_send::<LLMResult>();
        assert_sync::<LLMResult>();
        assert_send::<ChatResult>();
        assert_sync::<ChatResult>();
        assert_send::<GenerationChunk>();
        assert_sync::<GenerationChunk>();
        assert_send::<ChatGenerationChunk>();
        assert_sync::<ChatGenerationChunk>();
    }

    #[test]
    fn test_llm_result_flatten_single() {
        let g = Generation::new("only");
        let result = LLMResult::new(vec![vec![g]]);
        let flat = result.flatten();
        assert_eq!(flat.len(), 1);
    }

    #[test]
    fn test_llm_result_flatten_shared_llm_output() {
        let g1 = Generation::new("a");
        let g2 = Generation::new("b");
        let g3 = Generation::new("c");
        let result = LLMResult::new(vec![vec![g1], vec![g2], vec![g3]])
            .with_llm_output(serde_json::json!({"model": "gpt-4"}));
        let flat = result.flatten();
        assert_eq!(flat.len(), 3);
        assert!(flat[0].llm_output.is_some());
        assert!(flat[1].llm_output.is_some());
        assert!(flat[2].llm_output.is_some());
    }

    #[test]
    fn test_chat_generation_chunk_merge_empty_info() {
        let a = ChatGenerationChunk::from_text("hello", MessageType::AI);
        let b = ChatGenerationChunk::from_text(" world", MessageType::AI);
        let mut a_clone = a.clone();
        a_clone.merge(&b);
        assert_eq!(a_clone.text, "hello world");
        assert!(a_clone.generation_info.is_none());
    }

    #[test]
    fn test_chat_generation_chunk_merge_info_from_left() {
        let a = ChatGenerationChunk::from_text("hello", MessageType::AI)
            .with_generation_info(serde_json::json!({"a": 1}));
        let b = ChatGenerationChunk::from_text(" world", MessageType::AI);
        let c = a + b;
        assert_eq!(c.text, "hello world");
        assert_eq!(c.generation_info.unwrap()["a"], 1);
    }

    #[test]
    fn test_generation_serde_roundtrip() {
        let g = Generation::new("test")
            .with_generation_info(serde_json::json!({"token": 42}));
        let json = serde_json::to_string(&g).unwrap();
        let back: Generation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.text, "test");
        assert_eq!(back.generation_info.unwrap()["token"], 42);
    }

    #[test]
    fn test_chat_generation_serde_roundtrip() {
        let msg = BaseMessage::new("chat", MessageType::AI);
        let cg = ChatGeneration::new(msg)
            .with_generation_info(serde_json::json!({"finish": "stop"}));
        let json = serde_json::to_string(&cg).unwrap();
        let back: ChatGeneration = serde_json::from_str(&json).unwrap();
        assert_eq!(back.text, "chat");
        assert_eq!(back.generation_info.unwrap()["finish"], "stop");
    }

    #[test]
    fn test_chat_generation_chunk_from_text_types() {
        for mt in &[MessageType::Human, MessageType::AI, MessageType::System] {
            let chunk = ChatGenerationChunk::from_text("text", mt.clone());
            assert_eq!(chunk.text, "text");
            assert_eq!(chunk.message.content, "text");
        }
    }

    #[test]
    fn test_merge_chat_generation_chunks_multiple() {
        let chunks: Vec<ChatGenerationChunk> = vec!["a", " ", "b", " ", "c"]
            .into_iter()
            .map(|s| ChatGenerationChunk::from_text(s, MessageType::AI))
            .collect();
        let merged = merge_chat_generation_chunks(&chunks);
        assert!(merged.is_some());
        assert_eq!(merged.unwrap().text, "a b c");
    }

    #[test]
    fn test_chat_generation_chunk_add_overload() {
        let a = ChatGenerationChunk::from_text("hello", MessageType::AI);
        let b = ChatGenerationChunk::from_text(" world", MessageType::AI);
        let c = a + b;
        assert_eq!(c.text, "hello world");
        assert_eq!(c.message.content, "hello world");
    }
}
