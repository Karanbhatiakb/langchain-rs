//! Error types and result alias for the LangChain framework.
//!
//! Defines [`ChainError`] — a unified error enum covering LLM, parser, tool,
//! prompt, vector store, embedding, agent, memory, I/O, validation, timeout,
//! rate-limit, config, callback, serialization, and stream errors — plus a
//! convenience [`Result`] alias.

use serde_json;
use std::io;
use thiserror::Error;

/// Unified error type for all LangChain operations.
///
/// Each variant carries a human-readable description.  Conversions from
/// [`serde_json::Error`] and [`std::io::Error`] are provided so that `?`
/// works naturally with those types.
#[derive(Error, Debug, Clone)]
pub enum ChainError {
    /// An error returned by an LLM provider.
    #[error("LLM error: {0}")]
    LLMError(String),
    /// An error during output parsing.
    #[error("Parser error: {0}")]
    ParserError(String),
    /// An error returned by a tool.
    #[error("Tool error: {0}")]
    ToolError(String),
    /// An error during prompt construction.
    #[error("Prompt error: {0}")]
    PromptError(String),
    /// An error from a vector store operation.
    #[error("Vector store error: {0}")]
    VectorStoreError(String),
    /// An error from an embedding operation.
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    /// An error from an agent operation.
    #[error("Agent error: {0}")]
    AgentError(String),
    /// An error from a memory operation.
    #[error("Memory error: {0}")]
    MemoryError(String),
    /// An I/O error (wraps [`std::io::Error`]).
    #[error("IO error: {0}")]
    IOError(String),
    /// A validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),
    /// A timeout occurred.
    #[error("Timeout")]
    TimeoutError,
    /// A rate limit was exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    /// A requested feature is not implemented.
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    /// A configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),
    /// An error from a callback handler.
    #[error("Callback error: {0}")]
    CallbackError(String),
    /// A serialization/deserialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// An error during streaming.
    #[error("Stream error: {0}")]
    StreamError(String),
}

impl From<serde_json::Error> for ChainError {
    fn from(e: serde_json::Error) -> Self {
        ChainError::ParserError(e.to_string())
    }
}

impl From<io::Error> for ChainError {
    fn from(e: io::Error) -> Self {
        ChainError::IOError(e.to_string())
    }
}

/// Convenience alias for `std::result::Result<T, ChainError>`.
pub type Result<T> = std::result::Result<T, ChainError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_error_display_llm() {
        let err = ChainError::LLMError("timeout".into());
        assert_eq!(format!("{}", err), "LLM error: timeout");
    }

    #[test]
    fn test_chain_error_display_parser() {
        let err = ChainError::ParserError("invalid json".into());
        assert_eq!(format!("{}", err), "Parser error: invalid json");
    }

    #[test]
    fn test_chain_error_display_tool() {
        let err = ChainError::ToolError("not found".into());
        assert_eq!(format!("{}", err), "Tool error: not found");
    }

    #[test]
    fn test_chain_error_display_prompt() {
        let err = ChainError::PromptError("missing var".into());
        assert_eq!(format!("{}", err), "Prompt error: missing var");
    }

    #[test]
    fn test_chain_error_display_vectorstore() {
        let err = ChainError::VectorStoreError("connection failed".into());
        assert_eq!(format!("{}", err), "Vector store error: connection failed");
    }

    #[test]
    fn test_chain_error_display_embedding() {
        let err = ChainError::EmbeddingError("model not found".into());
        assert_eq!(format!("{}", err), "Embedding error: model not found");
    }

    #[test]
    fn test_chain_error_display_agent() {
        let err = ChainError::AgentError("invalid action".into());
        assert_eq!(format!("{}", err), "Agent error: invalid action");
    }

    #[test]
    fn test_chain_error_display_memory() {
        let err = ChainError::MemoryError("buffer full".into());
        assert_eq!(format!("{}", err), "Memory error: buffer full");
    }

    #[test]
    fn test_chain_error_display_io() {
        let err = ChainError::IOError("file not found".into());
        assert_eq!(format!("{}", err), "IO error: file not found");
    }

    #[test]
    fn test_chain_error_display_validation() {
        let err = ChainError::ValidationError("bad input".into());
        assert_eq!(format!("{}", err), "Validation error: bad input");
    }

    #[test]
    fn test_chain_error_display_timeout() {
        let err = ChainError::TimeoutError;
        assert_eq!(format!("{}", err), "Timeout");
    }

    #[test]
    fn test_chain_error_display_rate_limit() {
        let err = ChainError::RateLimitError("too fast".into());
        assert_eq!(format!("{}", err), "Rate limit exceeded: too fast");
    }

    #[test]
    fn test_chain_error_from_serde_json() {
        let serde_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let chain_err: ChainError = serde_err.into();
        assert!(matches!(chain_err, ChainError::ParserError(_)));
    }

    #[test]
    fn test_chain_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
        let chain_err: ChainError = io_err.into();
        assert!(matches!(chain_err, ChainError::IOError(_)));
    }

    #[test]
    fn test_chain_error_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ChainError>();
        assert_sync::<ChainError>();
    }

    #[test]
    fn test_chain_error_clone() {
        let err = ChainError::LLMError("original".into());
        let cloned = err.clone();
        assert_eq!(format!("{}", cloned), "LLM error: original");
    }
}
