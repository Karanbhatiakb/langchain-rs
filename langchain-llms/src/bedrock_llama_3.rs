//! BedrockLlama3LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "bedrock_llama_3")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// BedrockLlama3LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockLlama3LLM {
    /// API key for authentication.
    pub api_key: String,
    /// Model identifier.
    pub model: String,
    /// Sampling temperature.
    pub temperature: Option<f64>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Base URL for API requests.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
    /// Provider-specific configuration.
    pub provider_specific: Value,
}

impl BedrockLlama3LLM {
    /// Creates a new `BedrockLlama3LLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://bedrock-runtime.us-east-1.amazonaws.com".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for BedrockLlama3LLM {
    fn provider_name(&self) -> &'static str {
        "bedrock_llama_3"
    }
}

impl ProviderConfig for BedrockLlama3LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
