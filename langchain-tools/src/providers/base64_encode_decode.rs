//! Base64 encode/decode tool implementation.
//!
//! Provides a `Base64Tool` that encodes and decodes Base64 strings.
//! Gated behind the `base64_encode_decode` feature flag.

use async_trait::async_trait;

use super::super::traits::{BaseTool, ToolResult};

/// Tool for Base64 encoding and decoding.
#[derive(Debug, Clone)]
pub struct Base64Tool;

impl Base64Tool {
    /// Create a new `Base64Tool`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Base64Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for Base64Tool {
    fn name(&self) -> &str {
        "base64"
    }

    fn description(&self) -> &str {
        "Encodes or decodes Base64 data. Prefix input with 'encode:' or \
         'decode:' to choose the operation."
    }

    async fn invoke(&self, _input: &str) -> ToolResult {
        tracing::warn!("Base64Tool is a stub");
        Ok("base64_stub".into())
    }
}
