//! DeepSeek Coder v2 LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "deepseek_coder_v2")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// DeepSeek Coder v2 LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekCoderV2LLM {
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

impl DeepSeekCoderV2LLM {
    /// Creates a new `DeepSeekCoderV2LLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://api.deepseek.com/v1".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for DeepSeekCoderV2LLM {
    fn provider_name(&self) -> &'static str {
        "deepseek_coder_v2"
    }
}

impl ProviderConfig for DeepSeekCoderV2LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
