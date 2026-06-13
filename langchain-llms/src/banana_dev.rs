//! BananaDevLLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "banana_dev")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// BananaDevLLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BananaDevLLM {
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

impl BananaDevLLM {
    /// Creates a new `BananaDevLLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://api.banana.dev/v1".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for BananaDevLLM {
    fn provider_name(&self) -> &'static str {
        "banana_dev"
    }
}

impl ProviderConfig for BananaDevLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
