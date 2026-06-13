//! Ai21JambaLLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "ai21_jamba")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Ai21JambaLLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ai21JambaLLM {
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

impl Ai21JambaLLM {
    /// Creates a new `Ai21JambaLLM` instance.
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

impl LLMProvider for Ai21JambaLLM {
    fn provider_name(&self) -> &'static str {
        "ai21_jamba"
    }
}

impl ProviderConfig for Ai21JambaLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
