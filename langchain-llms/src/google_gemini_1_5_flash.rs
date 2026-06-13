//! Google Gemini 1.5 Flash LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "google_gemini_1_5_flash")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Google Gemini 1.5 Flash LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGemini15FlashLLM {
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

impl GoogleGemini15FlashLLM {
    /// Creates a new `GoogleGemini15FlashLLM` instance.
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

impl LLMProvider for GoogleGemini15FlashLLM {
    fn provider_name(&self) -> &'static str {
        "google_gemini_1_5_flash"
    }
}

impl ProviderConfig for GoogleGemini15FlashLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
