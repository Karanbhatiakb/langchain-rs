//! BedrockMistralSmallLLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "bedrock_mistral_small")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// BedrockMistralSmallLLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockMistralSmallLLM {
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

impl BedrockMistralSmallLLM {
    /// Creates a new `BedrockMistralSmallLLM` instance.
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

impl LLMProvider for BedrockMistralSmallLLM {
    fn provider_name(&self) -> &'static str {
        "bedrock_mistral_small"
    }
}

impl ProviderConfig for BedrockMistralSmallLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
