//! FileInputChain — loads a file and processes its contents.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::Chain;

/// A chain that reads a file from disk and passes its contents downstream for
/// further processing.
pub struct FileInputChain;

impl FileInputChain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileInputChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Chain for FileInputChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["file_path".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["file_contents".to_string(), "file_path".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let _path = inputs
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut result = HashMap::new();
        result.insert(
            "file_contents".to_string(),
            Value::String(format!("Contents of '{}' (stub)", _path)),
        );
        result.insert("file_path".to_string(), Value::String(_path.to_string()));
        Ok(result)
    }
}
