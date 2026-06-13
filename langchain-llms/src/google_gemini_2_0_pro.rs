//! Google Gemini 2.0 Pro LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "google_gemini_2_0_pro")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Google Gemini 2.0 Pro LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGemini20ProLLM {
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

impl GoogleGemini20ProLLM {
    /// Creates a new `GoogleGemini20ProLLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://generativelanguage.googleapis.com/v1".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for GoogleGemini20ProLLM {
    fn provider_name(&self) -> &'static str {
        "google_gemini_2_0_pro"
    }
}

impl ProviderConfig for GoogleGemini20ProLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
