//! BedrockAi21JurassicLLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "bedrock_ai21_jurassic")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// BedrockAi21JurassicLLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockAi21JurassicLLM {
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

impl BedrockAi21JurassicLLM {
    /// Creates a new `BedrockAi21JurassicLLM` instance.
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

impl LLMProvider for BedrockAi21JurassicLLM {
    fn provider_name(&self) -> &'static str {
        "bedrock_ai21_jurassic"
    }
}

impl ProviderConfig for BedrockAi21JurassicLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
