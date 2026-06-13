//! Output parsers that transform raw LLM responses into structured types.
//!
//! Provides the [`OutputParser`] trait and implementations for string, JSON,
//! comma-separated, datetime, structured, enum, Pydantic, XML, regex, YAML,
//! and retry parsing.

use crate::errors::*;
use crate::errors::ChainError;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

/// Trait for parsing LLM output strings into structured types.
///
/// # Type parameters
/// * `T` — The target output type (must implement `DeserializeOwned + Serialize + Send`).
pub trait OutputParser<T: DeserializeOwned + Serialize + Send>: Send + Sync {
    /// Parses a single text string into type `T`.
    fn parse(&self, text: &str) -> Result<T>;
    /// Returns a string describing the expected output format.
    fn get_format_instructions(&self) -> String;
    /// Tries parsing multiple texts and returns the first success, or an error.
    fn parse_result(&self, texts: &[&str]) -> Result<T> {
        texts
            .iter()
            .find_map(|t| self.parse(t).ok())
            .ok_or_else(|| ChainError::ParserError("No valid result found".into()))
    }
}

/// An output parser that returns the input text unchanged as a `String`.
#[derive(Debug, Clone)]
pub struct StrOutputParser;

impl OutputParser<String> for StrOutputParser {
    fn parse(&self, text: &str) -> Result<String> {
        Ok(text.to_string())
    }

    fn get_format_instructions(&self) -> String {
        "Return a plain text string.".to_string()
    }
}

/// An output parser that deserializes JSON into the target type `T`.
#[derive(Debug, Clone)]
pub struct JsonOutputParser<T> {
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize> JsonOutputParser<T> {
    /// Creates a new `JsonOutputParser`.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> OutputParser<T> for JsonOutputParser<T> {
    fn parse(&self, text: &str) -> Result<T> {
        let text = text.trim();
        let value: T = serde_json::from_str(text)?;
        Ok(value)
    }

    fn get_format_instructions(&self) -> String {
        "Return a JSON object matching the expected schema.".to_string()
    }
}

impl<T: Default + DeserializeOwned + Serialize + Send + Sync> JsonOutputParser<T> {
    /// Attempts a partial parse, falling back to `T::default()` on failure.
    pub fn parse_partial(&self, text: &str) -> Result<T> {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
            if let Ok(parsed) = serde_json::from_value::<T>(v) {
                return Ok(parsed);
            }
        }
        Ok(T::default())
    }
}

/// Parses comma-separated text into a `Vec<String>`.
#[derive(Debug, Clone)]
pub struct CommaSeparatedListOutputParser;

impl OutputParser<Vec<String>> for CommaSeparatedListOutputParser {
    fn parse(&self, text: &str) -> Result<Vec<String>> {
        Ok(text.split(',').map(|s| s.trim().to_string()).collect())
    }

    fn get_format_instructions(&self) -> String {
        "Return a comma-separated list.".to_string()
    }
}

/// Parses datetime strings into `chrono::NaiveDateTime`.
///
/// Supports multiple ISO 8601 and common date formats.
#[derive(Debug, Clone)]
pub struct DatetimeOutputParser;

impl OutputParser<chrono::NaiveDateTime> for DatetimeOutputParser {
    fn parse(&self, text: &str) -> Result<chrono::NaiveDateTime> {
        let formats = &[
            "%Y-%m-%dT%H:%M:%S%.f%:z",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d",
        ];

        let trimmed = text.trim();

        for format in formats {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(trimmed, format) {
                return Ok(dt);
            }
        }

        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
            return Ok(dt.naive_utc());
        }

        Err(ChainError::ParserError(format!(
            "Could not parse datetime: {}",
            text
        )))
    }

    fn get_format_instructions(&self) -> String {
        "Return a datetime string in ISO 8601 format (e.g., 2024-01-01T00:00:00Z).".to_string()
    }
}

/// Parses structured JSON into the target type `T` (alias for `JsonOutputParser`).
#[derive(Debug, Clone)]
pub struct StructuredOutputParser<T> {
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize> StructuredOutputParser<T> {
    /// Creates a new `StructuredOutputParser`.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> OutputParser<T>
    for StructuredOutputParser<T>
{
    fn parse(&self, text: &str) -> Result<T> {
        let text = text.trim();
        let value: T = serde_json::from_str(text).map_err(|e| {
            ChainError::ParserError(format!("Structured parse error: {}", e))
        })?;
        Ok(value)
    }

    fn get_format_instructions(&self) -> String {
        "Return a structured JSON object matching the expected schema.".to_string()
    }
}

/// Parses an enum variant from a text string.
#[derive(Debug, Clone)]
pub struct EnumOutputParser<T> {
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize> EnumOutputParser<T> {
    /// Creates a new `EnumOutputParser`.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> OutputParser<T> for EnumOutputParser<T> {
    fn parse(&self, text: &str) -> Result<T> {
        let text = text.trim().trim_matches('"').trim();
        serde_json::from_value(serde_json::Value::String(text.to_string())).map_err(|e| {
            ChainError::ParserError(format!("Enum parse error: {}", e))
        })
    }

    fn get_format_instructions(&self) -> String {
        "Return one of the valid enum values.".to_string()
    }
}

/// Parses a typed value from JSON, cleaning markdown code fences if present.
///
/// Uses [`schemars::JsonSchema`] to generate format instructions.
#[derive(Debug, Clone)]
pub struct PydanticOutputParser<T> {
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize> PydanticOutputParser<T> {
    /// Creates a new `PydanticOutputParser`.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + schemars::JsonSchema + Send + Sync> OutputParser<T>
    for PydanticOutputParser<T>
{
    fn parse(&self, text: &str) -> Result<T> {
        let cleaned = text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        let value: T = serde_json::from_str(cleaned)?;
        Ok(value)
    }

    fn get_format_instructions(&self) -> String {
        let schema = schemars::schema_for!(T);
        format!(
            "Return a JSON object matching this schema:\n{}",
            serde_json::to_string_pretty(&schema).unwrap_or_default()
        )
    }
}

/// Parses simple `<tag>value</tag>` XML into a JSON object.
#[derive(Debug, Clone)]
pub struct XMLOutputParser;

impl OutputParser<serde_json::Value> for XMLOutputParser {
    fn parse(&self, text: &str) -> Result<serde_json::Value> {
        let re = Regex::new(r"<(\w+)>([^<]*)</\1>").unwrap();
        let mut map = HashMap::new();
        for cap in re.captures_iter(text) {
            let key = cap.get(1).unwrap().as_str().to_string();
            let value = cap.get(2).unwrap().as_str().to_string();
            map.insert(key, serde_json::Value::String(value));
        }
        Ok(serde_json::to_value(map)?)
    }

    fn get_format_instructions(&self) -> String {
        "Return XML with tags like <tag>value</tag>.".to_string()
    }
}

/// Parses text using a configurable regex, extracting named or numbered groups
/// into a `HashMap<String, String>`.
#[derive(Debug, Clone)]
pub struct RegexOutputParser {
    /// The regex pattern to match against.
    pub regex: Regex,
    /// Names for captured groups (optional; defaults to `group_0`, `group_1`, …).
    pub output_keys: Vec<String>,
}

impl RegexOutputParser {
    /// Creates a new `RegexOutputParser`.
    pub fn new(regex: &str) -> Self {
        Self {
            regex: Regex::new(regex).unwrap(),
            output_keys: Vec::new(),
        }
    }

    /// Sets named output keys for capture groups (builder pattern).
    pub fn with_keys(mut self, keys: Vec<String>) -> Self {
        self.output_keys = keys;
        self
    }
}

impl OutputParser<HashMap<String, String>> for RegexOutputParser {
    fn parse(&self, text: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        if let Some(caps) = self.regex.captures(text) {
            if self.output_keys.is_empty() {
                for (i, cap) in caps.iter().enumerate().skip(1) {
                    if let Some(m) = cap {
                        result.insert(format!("group_{}", i), m.as_str().to_string());
                    }
                }
            } else {
                for (i, key) in self.output_keys.iter().enumerate() {
                    if let Some(cap) = caps.get(i + 1) {
                        result.insert(key.clone(), cap.as_str().to_string());
                    }
                }
            }
        }
        Ok(result)
    }

    fn get_format_instructions(&self) -> String {
        format!("Return text matching the regex pattern: {}", self.regex)
    }
}

/// Parses YAML text into the target type `T`.
#[derive(Debug, Clone)]
pub struct YamlOutputParser<T> {
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize> YamlOutputParser<T> {
    /// Creates a new `YamlOutputParser`.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> OutputParser<T> for YamlOutputParser<T> {
    fn parse(&self, text: &str) -> Result<T> {
        let value: T = serde_yaml::from_str(text).map_err(|e| {
            ChainError::ParserError(format!("YAML parse error: {}", e))
        })?;
        Ok(value)
    }

    fn get_format_instructions(&self) -> String {
        "Return a YAML object matching the expected schema.".to_string()
    }
}

/// An output parser that retries parsing up to `max_retries` times.
///
/// Useful when dealing with nondeterministic parsing or external services.
pub struct RetryOutputParser<T> {
    inner: Box<dyn OutputParser<T>>,
    /// Maximum number of parse attempts.
    pub max_retries: usize,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize + Send + Sync> RetryOutputParser<T> {
    /// Creates a new `RetryOutputParser`.
    pub fn new(inner: Box<dyn OutputParser<T>>, max_retries: usize) -> Self {
        Self {
            inner,
            max_retries,
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> std::fmt::Debug for RetryOutputParser<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetryOutputParser")
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> OutputParser<T> for RetryOutputParser<T> {
    fn parse(&self, text: &str) -> Result<T> {
        let mut last_error = None;
        for _ in 0..self.max_retries {
            match self.inner.parse(text) {
                Ok(result) => return Ok(result),
                Err(e) => last_error = Some(e),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            ChainError::ParserError("Retry output parser exhausted".into())
        }))
    }

    fn get_format_instructions(&self) -> String {
        self.inner.get_format_instructions()
    }
}

/// Parses OpenAI function call responses into a JSON value.
///
/// Attempts to extract the `"function_call"` key from a JSON object;
/// falls back to parsing the entire text as JSON if the key is absent.
#[derive(Debug, Clone)]
pub struct OpenAIFunctionsOutputParser;

impl OutputParser<serde_json::Value> for OpenAIFunctionsOutputParser {
    fn parse(&self, text: &str) -> Result<serde_json::Value> {
        let trimmed = text.trim();
        let value: serde_json::Value = serde_json::from_str(trimmed)?;
        if let Some(func_call) = value.get("function_call") {
            Ok(func_call.clone())
        } else {
            Ok(value)
        }
    }

    fn get_format_instructions(&self) -> String {
        "Return a JSON object with a 'function_call' key containing 'name' and 'arguments'.".to_string()
    }
}

/// Parses OpenAI tool call responses into a JSON value.
///
/// Attempts to extract the `"tool_calls"` key from a JSON object;
/// falls back to parsing the entire text as JSON if the key is absent.
#[derive(Debug, Clone)]
pub struct OpenAIToolsOutputParser;

impl OutputParser<serde_json::Value> for OpenAIToolsOutputParser {
    fn parse(&self, text: &str) -> Result<serde_json::Value> {
        let trimmed = text.trim();
        let value: serde_json::Value = serde_json::from_str(trimmed)?;
        if let Some(tool_calls) = value.get("tool_calls") {
            Ok(tool_calls.clone())
        } else {
            Ok(value)
        }
    }

    fn get_format_instructions(&self) -> String {
        "Return a JSON object with a 'tool_calls' array.".to_string()
    }
}

/// Parses JSON from the `"arguments"` field of a function call.
///
/// If the input is a JSON object containing `"arguments"`, that field is
/// parsed as JSON; otherwise the entire text is parsed as JSON.
#[derive(Debug, Clone)]
pub struct JsonOutputFunctionsParser;

impl OutputParser<serde_json::Value> for JsonOutputFunctionsParser {
    fn parse(&self, text: &str) -> Result<serde_json::Value> {
        let trimmed = text.trim();
        let value: serde_json::Value = serde_json::from_str(trimmed)?;
        if let Some(args) = value.get("arguments") {
            if args.is_string() {
                let parsed: serde_json::Value = serde_json::from_str(args.as_str().unwrap_or(""))?;
                return Ok(parsed);
            }
            return Ok(args.clone());
        }
        Ok(value)
    }

    fn get_format_instructions(&self) -> String {
        "Return a JSON object inside a function call's 'arguments' field.".to_string()
    }
}

/// Parses JSON from the `"function.arguments"` field of each tool call.
///
/// Returns a `Vec<serde_json::Value>` where each element is the parsed
/// `"arguments"` from a tool call. Falls back to parsing the whole text
/// as a single-element vec.
#[derive(Debug, Clone)]
pub struct JsonOutputToolsParser;

impl OutputParser<Vec<serde_json::Value>> for JsonOutputToolsParser {
    fn parse(&self, text: &str) -> Result<Vec<serde_json::Value>> {
        let trimmed = text.trim();
        let value: serde_json::Value = serde_json::from_str(trimmed)?;
        if let Some(tool_calls) = value.get("tool_calls") {
            if let Some(calls) = tool_calls.as_array() {
                let mut results = Vec::new();
                for call in calls {
                    if let Some(func) = call.get("function") {
                        if let Some(args) = func.get("arguments") {
                            if args.is_string() {
                                let parsed: serde_json::Value =
                                    serde_json::from_str(args.as_str().unwrap_or(""))?;
                                results.push(parsed);
                            } else {
                                results.push(args.clone());
                            }
                        }
                    }
                }
                if !results.is_empty() {
                    return Ok(results);
                }
            }
        }
        Ok(vec![value])
    }

    fn get_format_instructions(&self) -> String {
        "Return a JSON object with 'tool_calls' array where each element has 'function.arguments'.".to_string()
    }
}

/// Wraps another parser and an LLM-like retry function.
///
/// On parse failure, calls the `retry_fn` with the original text and the
/// error message, then retries parsing up to `max_retries` times.
pub struct OutputFixingParser<T: DeserializeOwned + Serialize + Send> {
    /// The inner parser to attempt first.
    pub parser: Arc<dyn OutputParser<T>>,
    /// Async function that produces a corrected text given the original and the error.
    pub retry_fn: Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String>> + Send>> + Send + Sync>,
    /// Maximum number of retry attempts.
    pub max_retries: usize,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize + Send + Sync> std::fmt::Debug for OutputFixingParser<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputFixingParser")
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> Clone for OutputFixingParser<T> {
    fn clone(&self) -> Self {
        Self {
            parser: Arc::clone(&self.parser),
            retry_fn: Arc::clone(&self.retry_fn),
            max_retries: self.max_retries,
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> OutputFixingParser<T> {
    /// Creates a new `OutputFixingParser`.
    pub fn new(
        parser: Arc<dyn OutputParser<T>>,
        retry_fn: Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String>> + Send>> + Send + Sync>,
        max_retries: usize,
    ) -> Self {
        Self {
            parser,
            retry_fn,
            max_retries,
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> OutputParser<T> for OutputFixingParser<T> {
    fn parse(&self, text: &str) -> Result<T> {
        if let Ok(result) = self.parser.parse(text) {
            return Ok(result);
        }
        Err(ChainError::ParserError(
            "OutputFixingParser requires async runtime; use parse_with_retry instead".into(),
        ))
    }

    fn get_format_instructions(&self) -> String {
        self.parser.get_format_instructions()
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> OutputFixingParser<T> {
    /// Async retry parse: tries the inner parser, and on failure calls
    /// `retry_fn` with `"{original_text}\nError: {error}"`, then retries
    /// up to `max_retries` times.
    pub async fn parse_with_retry(&self, text: &str) -> Result<T> {
        if let Ok(result) = self.parser.parse(text) {
            return Ok(result);
        }
        let mut current_text = text.to_string();
        for _ in 0..self.max_retries {
            let err_msg = match self.parser.parse(&current_text) {
                Ok(result) => return Ok(result),
                Err(e) => e.to_string(),
            };
            let retry_input = format!("{}\nError: {}", current_text, err_msg);
            match (self.retry_fn)(retry_input).await {
                Ok(corrected) => {
                    current_text = corrected;
                    if let Ok(result) = self.parser.parse(&current_text) {
                        return Ok(result);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Err(ChainError::ParserError("OutputFixingParser: all retry attempts failed".into()))
    }
}

/// Applies a transformation function to parsed output.
///
/// Trims the input text first, then applies the configured transform function.
pub struct TransformOutputParser {
    /// The transformation function applied to trimmed text.
    pub transform: Arc<dyn Fn(String) -> Result<String> + Send + Sync>,
}

impl std::fmt::Debug for TransformOutputParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransformOutputParser").finish()
    }
}

impl Clone for TransformOutputParser {
    fn clone(&self) -> Self {
        Self {
            transform: Arc::clone(&self.transform),
        }
    }
}

impl TransformOutputParser {
    /// Creates a new `TransformOutputParser` with the given transform function.
    pub fn new(transform: Arc<dyn Fn(String) -> Result<String> + Send + Sync>) -> Self {
        Self { transform }
    }
}

impl OutputParser<String> for TransformOutputParser {
    fn parse(&self, text: &str) -> Result<String> {
        let trimmed = text.trim().to_string();
        (self.transform)(trimmed)
    }

    fn get_format_instructions(&self) -> String {
        "Return text that will be transformed by the configured transform function.".to_string()
    }
}

/// Parses markdown bullet lists into a `Vec<String>`.
///
/// Extracts lines starting with `- ` or `* ` and trims the prefix.
#[derive(Debug, Clone)]
pub struct MarkdownListOutputParser;

impl OutputParser<Vec<String>> for MarkdownListOutputParser {
    fn parse(&self, text: &str) -> Result<Vec<String>> {
        let items: Vec<String> = text
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("- ") {
                    Some(trimmed[2..].trim().to_string())
                } else if trimmed.starts_with("* ") {
                    Some(trimmed[2..].trim().to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(items)
    }

    fn get_format_instructions(&self) -> String {
        "Return a markdown bullet list with items starting with '- ' or '* '.".to_string()
    }
}

/// Parses numbered lists into a `Vec<String>`.
///
/// Extracts lines matching `N. ` or `N) ` patterns (where `N` is digits)
/// and trims the prefix.
#[derive(Debug, Clone)]
pub struct NumberedListOutputParser;

impl OutputParser<Vec<String>> for NumberedListOutputParser {
    fn parse(&self, text: &str) -> Result<Vec<String>> {
        let re = Regex::new(r"^\s*(\d+)[.)]\s+").map_err(|e| {
            ChainError::ParserError(format!("Regex compilation error: {}", e))
        })?;
        let items: Vec<String> = text
            .lines()
            .filter_map(|line| {
                if let Some(caps) = re.captures(line) {
                    let full_match = caps.get(0)?;
                    Some(line[full_match.end()..].trim().to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(items)
    }

    fn get_format_instructions(&self) -> String {
        "Return a numbered list with items starting with 'N. ' or 'N) '.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_openai_functions_output_parser() {
        let parser = OpenAIFunctionsOutputParser;
        let text = r#"{"function_call": {"name": "get_weather", "arguments": "{\"city\": \"SF\"}"}}"#;
        let result = parser.parse(text).unwrap();
        assert_eq!(result["name"], "get_weather");
        assert_eq!(result["arguments"], "{\"city\": \"SF\"}");
    }

    #[test]
    fn test_openai_functions_output_parser_no_function_call() {
        let parser = OpenAIFunctionsOutputParser;
        let text = r#"{"result": "hello"}"#;
        let result = parser.parse(text).unwrap();
        assert_eq!(result["result"], "hello");
    }

    #[test]
    fn test_openai_tools_output_parser() {
        let parser = OpenAIToolsOutputParser;
        let text = r#"{"tool_calls": [{"id": "1", "function": {"name": "search", "arguments": "{\"q\": \"rust\"}"}}]}"#;
        let result = parser.parse(text).unwrap();
        assert!(result.is_array());
        assert_eq!(result[0]["id"], "1");
    }

    #[test]
    fn test_openai_tools_output_parser_no_tool_calls() {
        let parser = OpenAIToolsOutputParser;
        let text = r#"{"message": "hi"}"#;
        let result = parser.parse(text).unwrap();
        assert_eq!(result["message"], "hi");
    }

    #[test]
    fn test_json_output_functions_parser() {
        let parser = JsonOutputFunctionsParser;
        let text = r#"{"arguments": "{\"key\": \"value\"}"}"#;
        let result = parser.parse(text).unwrap();
        assert_eq!(result["key"], "value");
    }

    #[test]
    fn test_json_output_functions_parser_object_args() {
        let parser = JsonOutputFunctionsParser;
        let text = r#"{"arguments": {"key": "value"}}"#;
        let result = parser.parse(text).unwrap();
        assert_eq!(result["key"], "value");
    }

    #[test]
    fn test_json_output_tools_parser() {
        let parser = JsonOutputToolsParser;
        let text = r#"{"tool_calls": [{"function": {"arguments": "{\"x\": 1}"}}, {"function": {"arguments": "{\"y\": 2}"}}]}"#;
        let result = parser.parse(text).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["x"], 1);
        assert_eq!(result[1]["y"], 2);
    }

    #[test]
    fn test_json_output_tools_parser_fallback() {
        let parser = JsonOutputToolsParser;
        let text = r#"{"key": "value"}"#;
        let result = parser.parse(text).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["key"], "value");
    }

    #[test]
    fn test_markdown_list_output_parser() {
        let parser = MarkdownListOutputParser;
        let text = "- item1\n- item2\n- item3";
        let result = parser.parse(text).unwrap();
        assert_eq!(result, vec!["item1", "item2", "item3"]);
    }

    #[test]
    fn test_markdown_list_output_parser_asterisk() {
        let parser = MarkdownListOutputParser;
        let text = "* alpha\n* beta";
        let result = parser.parse(text).unwrap();
        assert_eq!(result, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_numbered_list_output_parser() {
        let parser = NumberedListOutputParser;
        let text = "1. first\n2. second\n3. third";
        let result = parser.parse(text).unwrap();
        assert_eq!(result, vec!["first", "second", "third"]);
    }

    #[test]
    fn test_numbered_list_output_parser_paren() {
        let parser = NumberedListOutputParser;
        let text = "1) one\n2) two";
        let result = parser.parse(text).unwrap();
        assert_eq!(result, vec!["one", "two"]);
    }

    #[test]
    fn test_transform_output_parser() {
        let parser = TransformOutputParser::new(Arc::new(|s: String| Ok(s)));
        let result = parser.parse("  hello world  ").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_transform_output_parser_uppercase() {
        let parser = TransformOutputParser::new(Arc::new(|s: String| Ok(s.to_uppercase())));
        let result = parser.parse("hello").unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_output_fixing_parser_success() {
        let inner: Arc<dyn OutputParser<serde_json::Value>> = Arc::new(JsonOutputParser::new());
        let retry_fn = Arc::new(|_s: String| {
            Box::pin(async { Ok("{}".to_string()) })
                as Pin<Box<dyn Future<Output = Result<String>> + Send>>
        });
        let parser = OutputFixingParser::new(inner, retry_fn, 1);
        let result = parser.parse(r#"{"ok": true}"#).unwrap();
        assert_eq!(result["ok"], true);
    }

    #[test]
    fn test_str_output_parser() {
        let parser = StrOutputParser;
        let result = parser.parse("hello world").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_str_output_parser_empty() {
        let parser = StrOutputParser;
        let result = parser.parse("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_comma_separated_list_output_parser() {
        let parser = CommaSeparatedListOutputParser;
        let result = parser.parse("a, b, c").unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_comma_separated_list_output_parser_empty() {
        let parser = CommaSeparatedListOutputParser;
        let result = parser.parse("").unwrap();
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn test_comma_separated_list_output_parser_single() {
        let parser = CommaSeparatedListOutputParser;
        let result = parser.parse("single").unwrap();
        assert_eq!(result, vec!["single"]);
    }

    #[test]
    fn test_json_output_parser_valid() {
        let parser: JsonOutputParser<serde_json::Value> = JsonOutputParser::new();
        let result = parser.parse(r#"{"key": "value"}"#).unwrap();
        assert_eq!(result["key"], "value");
    }

    #[test]
    fn test_json_output_parser_invalid() {
        let parser: JsonOutputParser<serde_json::Value> = JsonOutputParser::new();
        let result = parser.parse("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_output_parser_empty() {
        let parser: JsonOutputParser<serde_json::Value> = JsonOutputParser::new();
        let result = parser.parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_markdown_list_output_parser_no_items() {
        let parser = MarkdownListOutputParser;
        let result = parser.parse("just text").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_numbered_list_output_parser_no_items() {
        let parser = NumberedListOutputParser;
        let result = parser.parse("plain text").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_transform_output_parser_trim() {
        let parser = TransformOutputParser::new(Arc::new(|s: String| Ok(s.trim().to_string())));
        let result = parser.parse("  spaced  ").unwrap();
        assert_eq!(result, "spaced");
    }

    #[test]
    fn test_output_parser_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<StrOutputParser>();
        assert_sync::<StrOutputParser>();
        assert_send::<CommaSeparatedListOutputParser>();
        assert_sync::<CommaSeparatedListOutputParser>();
        assert_send::<JsonOutputParser<serde_json::Value>>();
        assert_sync::<JsonOutputParser<serde_json::Value>>();
        assert_send::<MarkdownListOutputParser>();
        assert_sync::<MarkdownListOutputParser>();
        assert_send::<NumberedListOutputParser>();
        assert_sync::<NumberedListOutputParser>();
    }
}
