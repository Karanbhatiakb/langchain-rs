//! General-purpose utility functions used across the LangChain codebase.

use crate::errors::*;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// Retrieves a configuration value from a dict or environment variable.
///
/// Checks the env var `env_key` first; falls back to `default`. Returns a
/// [`ChainError::ValidationError`] if neither is available.
///
/// # Errors
/// Returns [`ChainError::ValidationError`] if the env var is not set and no
/// default is provided.
pub fn get_from_dict_or_env(
    key: &str,
    env_key: &str,
    default: Option<&str>,
) -> Result<String> {
    if let Ok(val) = std::env::var(env_key) {
        return Ok(val);
    }
    if let Some(val) = default {
        return Ok(val.to_string());
    }
    Err(ChainError::ValidationError(format!(
        "Missing {} and env var {}",
        key, env_key
    )))
}

/// Converts a [`serde_json::Value`] to its string representation.
///
/// Recursively handles nested arrays and objects.
pub fn stringify_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(stringify_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, stringify_value(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

/// Extracts and parses a JSON value from markdown fenced code blocks.
///
/// Looks for ` ```json ... ``` ` or ` ``` ... ``` ` blocks. If none is found,
/// attempts to parse the entire trimmed input as JSON.
///
/// # Errors
/// Returns [`ChainError::ParserError`] if JSON parsing fails.
pub fn parse_json_markdown(text: &str) -> Result<Value> {
    lazy_static! {
        static ref JSON_BLOCK_RE: Regex =
            Regex::new(r"(?s)```(?:json)?\s*([\s\S]*?)```").unwrap();
    }

    if let Some(caps) = JSON_BLOCK_RE.captures(text) {
        let inner = caps.get(1).unwrap().as_str().trim();
        return Ok(serde_json::from_str(inner)?);
    }

    Ok(serde_json::from_str(text.trim())?)
}

/// Computes the cosine similarity between two equal-length float slices.
///
/// Returns `0.0` if the slices differ in length, are empty, or either has
/// zero norm.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

/// Produces a simple list of token IDs from a text string.
///
/// Uses a rough heuristic of `text.len() / 4` as the token count.
/// Returns sequential IDs `0..token_count`.
pub fn get_token_ids(text: &str) -> Vec<u32> {
    let token_count = (text.len() as u32 + 3) / 4;
    (0..token_count).collect()
}

/// Safe division that returns `0.0` when `b == 0.0`.
pub fn safe_div(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        0.0
    } else {
        a / b
    }
}

/// Truncates `text` at the first occurrence of any stop token.
///
/// If none of the stop tokens are found, the original text is returned
/// unchanged.
pub fn enforce_stop_tokens(text: &str, stop_tokens: &[String]) -> String {
    let mut result = text.to_string();
    for token in stop_tokens {
        if let Some(pos) = result.find(token) {
            result.truncate(pos);
        }
    }
    result
}

/// Merges two JSON dictionaries. Values in `override_dict` take precedence.
pub fn merge_dicts(
    base: &HashMap<String, Value>,
    override_dict: &HashMap<String, Value>,
) -> HashMap<String, Value> {
    let mut merged = base.clone();
    for (k, v) in override_dict {
        merged.insert(k.clone(), v.clone());
    }
    merged
}

/// Flattens a nested JSON dict into a dot-separated key → string map.
///
/// Nested `Value::Object` entries produce combined keys like `"parent.child"`.
pub fn flatten_dicts(dict: &HashMap<String, Value>) -> HashMap<String, String> {
    let mut flat = HashMap::new();
    for (key, value) in dict {
        match value {
            Value::String(s) => {
                flat.insert(key.clone(), s.clone());
            }
            Value::Object(obj) => {
                for (sub_key, sub_val) in obj {
                    let combined_key = format!("{}.{}", key, sub_key);
                    if let Value::String(s) = sub_val {
                        flat.insert(combined_key, s.clone());
                    } else {
                        flat.insert(combined_key, sub_val.to_string());
                    }
                }
            }
            _ => {
                flat.insert(key.clone(), stringify_value(value));
            }
        }
    }
    flat
}

/// Blocks on an async future using a temporary Tokio runtime.
///
/// Useful for calling async code from synchronous contexts.
///
/// # Errors
/// Returns [`ChainError::LLMError`] if the Tokio runtime cannot be created or
/// the future itself fails.
pub fn block_on_sync<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        ChainError::LLMError(format!("Failed to create runtime: {}", e))
    })?;
    rt.block_on(future)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_get_token_ids() {
        let ids = get_token_ids("hello");
        // len 5 → ceil(5/4) = 2
        assert_eq!(ids.len(), 2);
        assert_eq!(ids, vec![0, 1]);
    }

    #[test]
    fn test_get_token_ids_empty() {
        let ids = get_token_ids("");
        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn test_get_token_ids_long() {
        let ids = get_token_ids("hello world this is a test string");
        assert_eq!(ids.len(), 9); // len 35 → ceil(35/4) = 9
    }

    #[test]
    fn test_safe_div_normal() {
        assert!((safe_div(10.0, 2.0) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_safe_div_zero_denominator() {
        assert!((safe_div(10.0, 0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_safe_div_both_zero() {
        assert!((safe_div(0.0, 0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_enforce_stop_tokens_single() {
        let text = "hello world stop here";
        let stops = vec!["stop".into()];
        assert_eq!(enforce_stop_tokens(text, &stops), "hello world ");
    }

    #[test]
    fn test_enforce_stop_tokens_no_stop() {
        let text = "hello world";
        let stops = vec!["stop".into()];
        assert_eq!(enforce_stop_tokens(text, &stops), "hello world");
    }

    #[test]
    fn test_enforce_stop_tokens_multiple() {
        let text = "hello stop and also halt";
        let stops = vec!["stop".into(), "halt".into()];
        let result = enforce_stop_tokens(text, &stops);
        assert_eq!(result, "hello ");
    }

    #[test]
    fn test_enforce_stop_tokens_empty() {
        let text = "hello";
        let stops: Vec<String> = vec![];
        assert_eq!(enforce_stop_tokens(text, &stops), "hello");
    }

    #[test]
    fn test_merge_dicts_no_override() {
        let mut base = HashMap::new();
        base.insert("a".into(), Value::String("1".into()));
        let overrides = HashMap::new();
        let merged = merge_dicts(&base, &overrides);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged.get("a").unwrap(), "1");
    }

    #[test]
    fn test_merge_dicts_with_override() {
        let mut base = HashMap::new();
        base.insert("a".into(), Value::String("old".into()));
        let mut overrides = HashMap::new();
        overrides.insert("a".into(), Value::String("new".into()));
        let merged = merge_dicts(&base, &overrides);
        assert_eq!(merged.get("a").unwrap(), "new");
    }

    #[test]
    fn test_merge_dicts_new_key() {
        let mut base = HashMap::new();
        base.insert("a".into(), Value::String("1".into()));
        let mut overrides = HashMap::new();
        overrides.insert("b".into(), Value::String("2".into()));
        let merged = merge_dicts(&base, &overrides);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_flatten_dicts_empty() {
        let dict = HashMap::new();
        let flat = flatten_dicts(&dict);
        assert!(flat.is_empty());
    }

    #[test]
    fn test_flatten_dicts_flat() {
        let mut dict = HashMap::new();
        dict.insert("key".into(), Value::String("val".into()));
        let flat = flatten_dicts(&dict);
        assert_eq!(flat.get("key").unwrap(), "val");
    }

    #[test]
    fn test_flatten_dicts_nested() {
        let mut inner = serde_json::Map::new();
        inner.insert("sub".into(), Value::String("v".into()));
        let mut dict = HashMap::new();
        dict.insert("parent".into(), Value::Object(inner));
        let flat = flatten_dicts(&dict);
        assert_eq!(flat.get("parent.sub").unwrap(), "v");
    }

    #[test]
    fn test_get_from_dict_or_env_uses_default() {
        let result = get_from_dict_or_env("key", "NONEXISTENT_ENV_XYZ", Some("default_val"));
        assert_eq!(result.unwrap(), "default_val");
    }

    #[test]
    fn test_get_from_dict_or_env_fails_without_default() {
        let result = get_from_dict_or_env("key", "NONEXISTENT_ENV_XYZ_2", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![1.0f32, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![0.0f32, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cosine_similarity_mismatched_length() {
        let a = vec![1.0f32, 0.0];
        let b = vec![1.0f32, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_from_dict_or_env_uses_env() {
        std::env::set_var("TEST_OPENCODE_KEY", "env_value");
        let result = get_from_dict_or_env("key", "TEST_OPENCODE_KEY", Some("fallback"));
        assert_eq!(result.unwrap(), "env_value");
        std::env::remove_var("TEST_OPENCODE_KEY");
    }

    #[test]
    fn test_merge_dicts_empty_base() {
        let base = HashMap::new();
        let mut overrides = HashMap::new();
        overrides.insert("k".into(), Value::String("v".into()));
        let merged = merge_dicts(&base, &overrides);
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_flatten_dicts_nested_object() {
        let mut inner = serde_json::Map::new();
        inner.insert("x".into(), Value::String("y".into()));
        let mut dict = HashMap::new();
        dict.insert("a".into(), Value::Object(inner));
        let flat = flatten_dicts(&dict);
        assert_eq!(flat.get("a.x").unwrap(), "y");
    }

    #[test]
    fn test_enforce_stop_tokens_first_wins() {
        let text = "hello stop and halt here";
        let stops = vec!["stop".into(), "halt".into()];
        let result = enforce_stop_tokens(text, &stops);
        assert_eq!(result, "hello ");
    }

    #[test]
    fn test_enforce_stop_tokens_empty_text() {
        let text = "";
        let stops = vec!["stop".into()];
        assert_eq!(enforce_stop_tokens(text, &stops), "");
    }

    #[test]
    fn test_block_on_sync_ok() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let called = Arc::new(AtomicBool::new(false));
        let _called = called.clone();
        let future = async move {
            _called.store(true, Ordering::SeqCst);
            Ok::<_, ChainError>("done".to_string())
        };
        let result = block_on_sync(future);
        assert!(result.is_ok());
        assert!(called.load(Ordering::SeqCst));
    }
}
