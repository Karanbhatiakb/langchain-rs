//! AbzuLLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "abzu")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// AbzuLLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbzuLLM {
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

impl AbzuLLM {
    /// Creates a new `AbzuLLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://api.abzu.ai/v1".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for AbzuLLM {
    fn provider_name(&self) -> &'static str {
        "abzu"
    }
}

impl ProviderConfig for AbzuLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
