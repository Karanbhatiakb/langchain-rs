//! WebInputChain — fetches a URL and processes its content.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that fetches content from a URL and makes it available for
/// downstream processing.
pub struct WebInputChain;

impl WebInputChain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WebInputChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for WebInputChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["url".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["web_content".to_string(), "url".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _url = inputs
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut result = HashMap::new();
        result.insert(
            "web_content".to_string(),
            Value::String(format!("Content fetched from '{}' (stub)", _url)),
        );
        result.insert("url".to_string(), Value::String(_url.to_string()));
        Ok(result)
    }
}
