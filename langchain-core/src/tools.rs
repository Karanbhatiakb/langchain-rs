//! Tool abstractions for the LangChain framework.
//!
//! Defines the [`BaseTool`] trait — the async interface that all LangChain
//! tools implement — plus supporting types for tool invocation, results,
//! and error handling.
//!
//! # Core types
//!
//! - [`BaseTool`] — async trait with `name`, `description`, `parameters`,
//!   `invoke`, `is_direct`, and `handle_tool_error`.
//! - [`ToolInvocation`] — pairs a tool input string with an optional call ID.
//! - [`ToolResult`] — convenience alias for `Result<String>`.
//! - [`ToolError`] — enum covering invalid-input, execution, and parsing
//!   errors.
//! - [`ToolException`] — structured exception that tools can raise to signal
//!   a recoverable error (mirroring the Python `ToolException`).

use crate::errors::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A single tool invocation request.
///
/// Carries the raw string input for the tool and an optional unique
/// identifier that links the invocation back to a model-generated
/// tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    /// The raw string input to be passed to the tool.
    pub tool_input: String,
    /// An optional identifier that links this invocation to a specific
    /// model-generated tool call.
    pub id: String,
}

impl ToolInvocation {
    /// Creates a new [`ToolInvocation`] with the given input and an empty id.
    pub fn new(tool_input: impl Into<String>) -> Self {
        Self {
            tool_input: tool_input.into(),
            id: String::new(),
        }
    }

    /// Sets the `id` field and returns `self` (builder pattern).
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
}

/// Convenience alias for the result of a tool invocation.
pub type ToolResult = Result<String>;

/// Errors that can occur during tool execution.
///
/// Each variant carries a human-readable description. Conversions from
/// [`ToolError`] to [`ChainError`] are provided so that `?` works
/// naturally in contexts that expect [`ChainError`].
#[derive(Error, Debug, Clone)]
pub enum ToolError {
    /// The input provided to the tool is invalid or malformed.
    #[error("Invalid tool input: {0}")]
    InvalidInput(String),
    /// An error occurred while the tool was executing.
    #[error("Tool execution error: {0}")]
    ExecutionError(String),
    /// The tool output could not be parsed into the expected format.
    #[error("Tool output parsing error: {0}")]
    ParsingError(String),
}

impl From<ToolError> for ChainError {
    fn from(e: ToolError) -> Self {
        ChainError::ToolError(e.to_string())
    }
}

/// A structured exception that tools can raise to signal a recoverable error.
///
/// When a tool raises a [`ToolException`], the agent or executor can decide
/// whether to propagate the error or handle it gracefully (e.g. by returning
/// a fallback observation). This mirrors the Python `ToolException` class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolException {
    /// Human-readable error message.
    pub message: String,
    /// Indicates whether the tool error should be surfaced to the agent
    /// as a fallback observation rather than aborting execution.
    pub send_to_llm: bool,
}

impl ToolException {
    /// Creates a new [`ToolException`] with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            send_to_llm: false,
        }
    }

    /// Sets `send_to_llm` to `true` and returns `self` (builder pattern).
    pub fn with_send_to_llm(mut self) -> Self {
        self.send_to_llm = true;
        self
    }
}

impl std::fmt::Display for ToolException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ToolException: {}", self.message)
    }
}

impl std::error::Error for ToolException {}





/// The base trait that all LangChain tools must implement.
///
/// Tools are components that agents can call to perform specific actions
/// (e.g. web search, calculator, database query). Every tool must provide:
///
/// - A unique [`name`](BaseTool::name).
/// - A [`description`](BaseTool::description) that tells the model when
///   and how to use the tool.
/// - Optional [`parameters`](BaseTool::parameters) as a JSON Schema value.
/// - An async [`invoke`](BaseTool::invoke) method that executes the tool.
///
/// Optional methods:
///
/// - [`is_direct`](BaseTool::is_direct) — return `true` to signal that the
///   tool's output should be returned directly to the user rather than
///   fed back into the agent loop.
/// - [`handle_tool_error`](BaseTool::handle_tool_error) — customise the
///   error message returned when the tool raises a [`ToolException`].
///
/// # Example
///
/// ```rust,no_run
/// use langchain_core::tools::{BaseTool, ToolResult};
/// use async_trait::async_trait;
///
/// struct EchoTool;
///
/// #[async_trait]
/// impl BaseTool for EchoTool {
///     fn name(&self) -> &str { "echo" }
///     fn description(&self) -> &str { "Returns the input unchanged." }
///     fn parameters(&self) -> Option<serde_json::Value> { None }
///     async fn invoke(&self, input: &str) -> ToolResult {
///         Ok(input.to_string())
///     }
/// }
/// ```
#[async_trait]
pub trait BaseTool: Send + Sync + 'static {
    /// Returns the unique name of the tool.
    ///
    /// The name is used by language models to select which tool to call,
    /// so it should be concise and descriptive (e.g. `"calculator"`,
    /// `"web_search"`).
    fn name(&self) -> &str;

    /// Returns a description of the tool that tells the model how, when,
    /// and why to use it.
    ///
    /// Good descriptions include the tool's purpose, the expected input
    /// format, and any constraints on usage.
    fn description(&self) -> &str;

    /// Returns the JSON Schema for the tool's input parameters, if any.
    ///
    /// When `Some(value)` is returned, `value` should be a valid JSON Schema
    /// object describing the tool's expected input. Returning `None` signals
    /// that the tool accepts a free-form string input.
    fn parameters(&self) -> Option<serde_json::Value>;

    /// Executes the tool with the given string input and returns the result.
    ///
    /// # Errors
    ///
    /// Returns a [`ToolResult`] (i.e. `Result<String, ChainError>`) on
    /// failure.
    async fn invoke(&self, input: &str) -> ToolResult;

    /// Returns `true` if the tool's output should be returned directly to
    /// the caller rather than being fed back into the agent loop.
    ///
    /// The default implementation returns `false`.
    fn is_direct(&self) -> bool {
        false
    }

    /// Produces a human-readable error message when the tool raises a
    /// [`ToolException`].
    ///
    /// The default implementation returns the exception's message, or a
    /// generic fallback string if the message is empty.
    fn handle_tool_error(&self, error: &str) -> String {
        if error.is_empty() {
            "Tool execution error".to_string()
        } else {
            error.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct UpperTool;

    #[async_trait]
    impl BaseTool for UpperTool {
        fn name(&self) -> &str {
            "upper"
        }
        fn description(&self) -> &str {
            "Converts the input string to uppercase."
        }
        fn parameters(&self) -> Option<serde_json::Value> {
            None
        }
        async fn invoke(&self, input: &str) -> ToolResult {
            Ok(input.to_uppercase())
        }
    }

    #[tokio::test]
    async fn test_base_tool_invoke() {
        let tool = UpperTool;
        assert_eq!(tool.name(), "upper");
        assert!(!tool.is_direct());
        let result = tool.invoke("hello").await.unwrap();
        assert_eq!(result, "HELLO");
    }

    #[tokio::test]
    async fn test_tool_invocation() {
        let inv = ToolInvocation::new("test input").with_id("call-123");
        assert_eq!(inv.tool_input, "test input");
        assert_eq!(inv.id, "call-123");
    }

    #[test]
    fn test_tool_error_conversion() {
        let err = ToolError::InvalidInput("bad input".into());
        let chain_err: ChainError = err.into();
        match chain_err {
            ChainError::ToolError(msg) => assert!(msg.contains("bad input")),
            _ => panic!("expected ToolError variant"),
        }
    }

    #[test]
    fn test_tool_exception() {
        let ex = ToolException::new("something went wrong").with_send_to_llm();
        assert!(ex.send_to_llm);
        assert_eq!(ex.to_string(), "ToolException: something went wrong");
    }

    #[test]
    fn test_handle_tool_error_default() {
        struct Dummy;
        #[async_trait]
        impl BaseTool for Dummy {
            fn name(&self) -> &str { "dummy" }
            fn description(&self) -> &str { "dummy" }
            fn parameters(&self) -> Option<serde_json::Value> { None }
            async fn invoke(&self, _input: &str) -> ToolResult { Ok(String::new()) }
        }
        let d = Dummy;
        assert_eq!(d.handle_tool_error("oops"), "oops");
        assert_eq!(d.handle_tool_error(""), "Tool execution error");
    }

    #[test]
    fn test_tool_invocation_default_id() {
        let inv = ToolInvocation::new("hello");
        assert!(inv.id.is_empty());
    }

    #[test]
    fn test_tool_exception_default_send_to_llm() {
        let ex = ToolException::new("error");
        assert!(!ex.send_to_llm);
    }

    #[test]
    fn test_tool_error_display() {
        let err = ToolError::InvalidInput("bad".into());
        let msg = format!("{}", err);
        assert_eq!(msg, "Invalid tool input: bad");

        let err = ToolError::ExecutionError("crash".into());
        let msg = format!("{}", err);
        assert_eq!(msg, "Tool execution error: crash");

        let err = ToolError::ParsingError("syntax".into());
        let msg = format!("{}", err);
        assert_eq!(msg, "Tool output parsing error: syntax");
    }

    #[test]
    fn test_tool_exception_display() {
        let ex = ToolException::new("fail");
        assert_eq!(format!("{}", ex), "ToolException: fail");
    }

    #[test]
    fn test_tool_exception_send_to_llm_flag() {
        let ex = ToolException::new("recoverable").with_send_to_llm();
        assert!(ex.send_to_llm);
        assert_eq!(ex.message, "recoverable");
    }

    #[test]
    fn test_tool_error_into_chain_error() {
        let err = ToolError::ExecutionError("crash".into());
        let chain: ChainError = err.into();
        match chain {
            ChainError::ToolError(msg) => assert!(msg.contains("crash")),
            _ => panic!("expected ToolError variant"),
        }
    }

    #[test]
    fn test_tool_exception_error_trait() {
        use std::error::Error;
        let ex = ToolException::new("oops");
        assert!(ex.source().is_none());
        assert_eq!(ex.to_string(), "ToolException: oops");
    }

    #[test]
    fn test_base_tool_defaults() {
        struct MinimalTool;
        #[async_trait]
        impl BaseTool for MinimalTool {
            fn name(&self) -> &str { "min" }
            fn description(&self) -> &str { "minimal" }
            fn parameters(&self) -> Option<serde_json::Value> { None }
            async fn invoke(&self, input: &str) -> ToolResult { Ok(input.into()) }
        }
        let t = MinimalTool;
        assert!(!t.is_direct());
        assert_eq!(t.handle_tool_error("err"), "err");
    }

    struct ReturnTool;
    #[async_trait]
    impl BaseTool for ReturnTool {
        fn name(&self) -> &str { "return" }
        fn description(&self) -> &str { "returns input" }
        fn parameters(&self) -> Option<serde_json::Value> { None }
        async fn invoke(&self, input: &str) -> ToolResult { Ok(input.into()) }
    }

    #[tokio::test]
    async fn test_base_tool_with_parameters() {
        struct ParamTool;
        #[async_trait]
        impl BaseTool for ParamTool {
            fn name(&self) -> &str { "param" }
            fn description(&self) -> &str { "has params" }
            fn parameters(&self) -> Option<serde_json::Value> {
                Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "x": {"type": "number"}
                    }
                }))
            }
            async fn invoke(&self, _input: &str) -> ToolResult { Ok("ok".into()) }
        }
        let t = ParamTool;
        let params = t.parameters().unwrap();
        assert_eq!(params["type"], "object");
    }

    #[tokio::test]
    async fn test_tool_invocation_empty_input() {
        let inv = ToolInvocation::new("").with_id("empty");
        assert_eq!(inv.tool_input, "");
        assert_eq!(inv.id, "empty");
    }

    #[test]
    fn test_base_tool_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<UpperTool>();
        assert_sync::<UpperTool>();
    }

    struct CalcTool;
    #[async_trait]
    impl BaseTool for CalcTool {
        fn name(&self) -> &str { "calc" }
        fn description(&self) -> &str { "Simple calculator" }
        fn parameters(&self) -> Option<serde_json::Value> {
            Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "expr": {"type": "string"}
                },
                "required": ["expr"]
            }))
        }
        async fn invoke(&self, input: &str) -> ToolResult {
            Ok(format!("result: {}", input))
        }
    }

    #[tokio::test]
    async fn test_tool_with_parameters_schema() {
        let tool = CalcTool;
        let params = tool.parameters().unwrap();
        assert_eq!(params["required"][0], "expr");
    }

    #[tokio::test]
    async fn test_tool_invoke_with_input() {
        let tool = CalcTool;
        let result = tool.invoke("2+2").await.unwrap();
        assert_eq!(result, "result: 2+2");
    }

    #[tokio::test]
    async fn test_tool_default_is_direct() {
        let tool = CalcTool;
        assert!(!tool.is_direct());
    }

    #[test]
    fn test_tool_exception_default_message() {
        let ex = ToolException::new("");
        assert_eq!(ex.message, "");
        assert!(!ex.send_to_llm);
    }

    #[test]
    fn test_tool_invocation_clone() {
        let inv = ToolInvocation::new("data").with_id("id1");
        let cloned = inv.clone();
        assert_eq!(cloned.tool_input, "data");
        assert_eq!(cloned.id, "id1");
    }
}
