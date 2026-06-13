use langchain_chains::sequential::SimpleSequentialChain;
use langchain_chains::transform::TransformChain;
use langchain_chains::types::Chain;
use langchain_core::prompt::PromptTemplate;
use langchain_core::output_parsers::{StrOutputParser, JsonOutputParser, OutputParser};
use langchain_core::errors::Result;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;

#[test]
fn test_str_output_parser() {
    let parser = StrOutputParser;
    let result: String = parser.parse("hello world").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_json_output_parser_valid() {
    let parser: JsonOutputParser<Value> = JsonOutputParser::new();
    let input = r#"{"key": "value"}"#;
    let result: Value = parser.parse(input).unwrap();
    assert_eq!(result["key"], "value");
}

#[test]
fn test_json_output_parser_invalid() {
    let parser: JsonOutputParser<Value> = JsonOutputParser::new();
    let input = "not json at all";
    let result: Result<Value> = parser.parse(input);
    assert!(result.is_err());
}

#[test]
fn test_json_output_parser_format_instructions() {
    let parser: JsonOutputParser<Value> = JsonOutputParser::new();
    let instructions = parser.get_format_instructions();
    assert!(!instructions.is_empty());
}

#[test]
fn test_prompt_template_single_var() {
    let prompt = PromptTemplate::from_template("Hello {name}!");
    let result = prompt.format(&HashMap::from([
        ("name".into(), "World".into()),
    ])).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_prompt_template_multiple_vars() {
    let prompt = PromptTemplate::from_template("{greeting} {name}!");
    let result = prompt.format(&HashMap::from([
        ("greeting".into(), "Hello".into()),
        ("name".into(), "World".into()),
    ])).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_prompt_template_missing_var() {
    let prompt = PromptTemplate::from_template("Hello {name} and {other}!");
    let result = prompt.format(&HashMap::from([
        ("name".into(), "World".into()),
    ]));
    assert!(result.is_err());
}

#[test]
fn test_sequential_chain_empty() {
    let chains: Vec<Arc<dyn Chain>> = vec![];
    let chain = SimpleSequentialChain::new(chains);
    assert!(chain.input_keys().is_empty());
    assert!(chain.output_keys().is_empty());
}

#[tokio::test]
async fn test_transform_chain() {
    let input_keys = vec!["input".to_string()];
    let output_keys = vec!["output".to_string()];
    let transform = Arc::new(|inputs: HashMap<String, Value>| -> Result<HashMap<String, Value>> {
        let mut out = HashMap::new();
        if let Some(v) = inputs.get("input") {
            if let Some(s) = v.as_str() {
                out.insert("output".into(), Value::String(s.to_uppercase()));
            }
        }
        Ok(out)
    });
    let chain = TransformChain::new(input_keys, output_keys, transform);
    assert_eq!(chain.input_keys(), vec!["input"]);
    assert_eq!(chain.output_keys(), vec!["output"]);

    let mut inputs = HashMap::new();
    inputs.insert("input".into(), Value::String("hello".into()));
    let result = chain.call(inputs).await.unwrap();
    assert_eq!(result.get("output").unwrap().as_str(), Some("HELLO"));
}

#[tokio::test]
async fn test_transform_chain_reverse() {
    let input_keys = vec!["text".to_string()];
    let output_keys = vec!["result".to_string()];
    let transform = Arc::new(|inputs: HashMap<String, Value>| -> Result<HashMap<String, Value>> {
        let mut out = HashMap::new();
        if let Some(v) = inputs.get("text") {
            if let Some(s) = v.as_str() {
                out.insert("result".into(), Value::String(s.chars().rev().collect()));
            }
        }
        Ok(out)
    });
    let chain = TransformChain::new(input_keys, output_keys, transform);
    let mut inputs = HashMap::new();
    inputs.insert("text".into(), Value::String("hello".into()));
    let result = chain.call(inputs).await.unwrap();
    assert_eq!(result.get("result").unwrap().as_str(), Some("olleh"));
}
