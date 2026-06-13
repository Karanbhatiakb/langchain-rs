use std::collections::HashMap;
use std::sync::Arc;

use langchain_core::callbacks::{CallbackManager, CallbackHandler, StdOutCallbackHandler, EmptyCallbackHandler};
use langchain_core::documents::Document;
use langchain_core::errors::ChainError;
use langchain_core::example_selectors::{ExampleSelector, LengthBasedExampleSelector};
use langchain_core::messages::{BaseMessage, HumanMessage, MessageType};
use langchain_core::output_parsers::{CommaSeparatedListOutputParser, JsonOutputParser, OutputParser, StrOutputParser};
use langchain_core::prompt::PromptTemplate;
use langchain_core::runnable::Runnable;
use langchain_core::utils::{stringify_value, parse_json_markdown};
use serde_json::Value;

#[tokio::test]
async fn test_base_message_new() {
    let msg = BaseMessage::new("hello", MessageType::Human);
    assert_eq!(msg.content, "hello");
    assert!(matches!(msg.message_type, MessageType::Human));
}

#[tokio::test]
async fn test_human_message_new() {
    let msg = HumanMessage::new("hello");
    assert_eq!(msg.content, "hello");
    assert!(matches!(msg.message_type, MessageType::Human));
}

#[tokio::test]
async fn test_message_deref() {
    let msg = HumanMessage::new("test content");
    assert_eq!(msg.content, "test content");
}

#[tokio::test]
async fn test_message_from_impls() {
    let human = HumanMessage::new("hello");
    let base: BaseMessage = human.into();
    assert_eq!(base.content, "hello");
    let back: HumanMessage = base.into();
    assert_eq!(back.content, "hello");
}

#[test]
fn test_chain_error_llm_error_display() {
    let err = ChainError::LLMError("rate limited".into());
    let display = format!("{}", err);
    assert_eq!(display, "LLM error: rate limited");
}

#[test]
fn test_chain_error_from_serde_json() {
    let invalid = r#"{"invalid": }"#;
    let result: Result<Value, ChainError> = serde_json::from_str(invalid).map_err(ChainError::from);
    assert!(result.is_err());
}

#[test]
fn test_chain_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let chain_err: ChainError = io_err.into();
    let display = format!("{}", chain_err);
    assert!(display.contains("IO error"));
}

#[test]
fn test_prompt_template_from_template() {
    let pt = PromptTemplate::from_template("Hello {name}, you are {age} years old");
    assert_eq!(pt.input_variables.len(), 2);
    assert!(pt.input_variables.contains(&"name".to_string()));
    assert!(pt.input_variables.contains(&"age".to_string()));
}

#[test]
fn test_prompt_template_format_single() {
    let pt = PromptTemplate::from_template("Hello {name}");
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "World".to_string());
    let result = pt.format(&vars).unwrap();
    assert_eq!(result, "Hello World");
}

#[test]
fn test_prompt_template_format_multiple() {
    let pt = PromptTemplate::from_template("{greeting}, {name}!");
    let mut vars = HashMap::new();
    vars.insert("greeting".to_string(), "Hello".to_string());
    vars.insert("name".to_string(), "Alice".to_string());
    let result = pt.format(&vars).unwrap();
    assert_eq!(result, "Hello, Alice!");
}

#[test]
fn test_prompt_template_format_missing_variable() {
    let pt = PromptTemplate::from_template("Hello {name}");
    let vars = HashMap::new();
    let result = pt.format(&vars);
    assert!(result.is_err());
}

#[test]
fn test_prompt_template_format_partial_variables() {
    let mut partials = HashMap::new();
    partials.insert("name".to_string(), "Bob".to_string());
    let pt = PromptTemplate::from_template("Hello {name}").with_partial(partials);
    let vars = HashMap::new();
    let result = pt.format(&vars).unwrap();
    assert_eq!(result, "Hello Bob");
}

#[test]
fn test_str_output_parser() {
    let parser = StrOutputParser;
    let result = parser.parse("hello world").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_json_output_parser_valid() {
    let parser: JsonOutputParser<HashMap<String, String>> = JsonOutputParser::new();
    let result = parser.parse(r#"{"key": "value"}"#).unwrap();
    assert_eq!(result.get("key").unwrap(), "value");
}

#[test]
fn test_json_output_parser_invalid() {
    let parser: JsonOutputParser<HashMap<String, String>> = JsonOutputParser::new();
    let result = parser.parse("not json");
    assert!(result.is_err());
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
    assert_eq!(result, vec![""]); // split on "" returns [""]
}

#[test]
fn test_comma_separated_list_output_parser_single() {
    let parser = CommaSeparatedListOutputParser;
    let result = parser.parse("single").unwrap();
    assert_eq!(result, vec!["single"]);
}

#[test]
fn test_document_new() {
    let doc = Document::new("content");
    assert_eq!(doc.page_content, "content");
    assert!(doc.metadata.is_empty());
    assert!(doc.score.is_none());
}

#[test]
fn test_document_with_metadata() {
    let mut meta = HashMap::new();
    meta.insert("key".to_string(), Value::String("val".to_string()));
    let doc = Document::new("content").with_metadata(meta);
    assert_eq!(doc.metadata.get("key").unwrap(), "val");
}

#[test]
fn test_document_with_score() {
    let doc = Document::new("content").with_score(0.95);
    assert_eq!(doc.score, Some(0.95));
}

struct TestRunnable;

#[async_trait::async_trait]
impl Runnable<i32, i32> for TestRunnable {
    async fn invoke(&self, input: i32) -> langchain_core::errors::Result<i32> {
        Ok(input * 2)
    }
}

#[tokio::test]
async fn test_runnable_invoke() {
    let runnable = TestRunnable;
    let result = runnable.invoke(5).await.unwrap();
    assert_eq!(result, 10);
}

#[tokio::test]
async fn test_runnable_batch() {
    let runnable = TestRunnable;
    let results = runnable.batch(vec![1, 2, 3]).await.unwrap();
    assert_eq!(results, vec![2, 4, 6]);
}

#[test]
fn test_callback_manager_new() {
    let manager = CallbackManager::new();
    let debug = format!("{:?}", manager);
    assert!(debug.contains("handler_count: 0"));
}

#[test]
fn test_callback_manager_with_handler() {
    let handler = Arc::new(EmptyCallbackHandler);
    let manager = CallbackManager::new().with_handler(handler);
    let debug = format!("{:?}", manager);
    assert!(debug.contains("handler_count: 1"));
}

#[test]
fn test_stdout_callback_handler() {
    let handler = StdOutCallbackHandler;
    let input = serde_json::json!({"text": "hello"});
    handler.on_chain_start("test", &input);
    handler.on_chain_end("test", &input);
    handler.on_llm_start("test", &["prompt".to_string()]);
    handler.on_llm_new_token("tok");
    handler.on_llm_end("test", &input);
    handler.on_tool_start("test", &input);
    handler.on_tool_end("test", &input);
    handler.on_text("hello");
    handler.on_agent_action(&input);
    handler.on_agent_finish(&input);
}

#[test]
fn test_stringify_value() {
    assert_eq!(stringify_value(&Value::String("hi".into())), "hi");
    assert_eq!(stringify_value(&Value::Number(42.into())), "42");
    assert_eq!(stringify_value(&Value::Null), "null");
    let arr = Value::Array(vec![Value::String("a".into()), Value::String("b".into())]);
    assert_eq!(stringify_value(&arr), "[a, b]");
    let mut map = serde_json::Map::new();
    map.insert("k".into(), Value::String("v".into()));
    let obj = Value::Object(map);
    assert_eq!(stringify_value(&obj), "{k: v}");
}

#[test]
fn test_parse_json_markdown_wrapped() {
    let input = "```json\n{\"key\": \"value\"}\n```";
    let result = parse_json_markdown(input).unwrap();
    assert_eq!(result, serde_json::json!({"key": "value"}));
}

#[test]
fn test_parse_json_markdown_plain() {
    let input = "{\"key\": \"value\"}";
    let result = parse_json_markdown(input).unwrap();
    assert_eq!(result, serde_json::json!({"key": "value"}));
}

#[tokio::test]
async fn test_length_based_example_selector() {
    let examples = vec![
        HashMap::from([("text".to_string(), Value::String("short".to_string()))]),
        HashMap::from([("text".to_string(), Value::String("longer text here".to_string()))]),
        HashMap::from([("text".to_string(), Value::String("another".to_string()))]),
    ];
    let selector = LengthBasedExampleSelector::new(examples, 20);
    let result = selector.select_examples(&HashMap::new()).await.unwrap();
    assert!(!result.is_empty());
}
