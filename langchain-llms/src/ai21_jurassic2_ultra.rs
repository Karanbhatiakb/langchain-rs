//! AI21 Jurassic-2 Ultra LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "ai21_jurassic2_ultra")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// AI21 Jurassic-2 Ultra LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ai21Jurassic2UltraLLM {
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

impl Ai21Jurassic2UltraLLM {
    /// Creates a new `Ai21Jurassic2UltraLLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://api.ai21.com/studio/v1".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for Ai21Jurassic2UltraLLM {
    fn provider_name(&self) -> &'static str {
        "ai21_jurassic2_ultra"
    }
}

impl ProviderConfig for Ai21Jurassic2UltraLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
