//! Ai21JambaInstructLLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "ai21_jamba_instruct")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Ai21JambaInstructLLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ai21JambaInstructLLM {
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

impl Ai21JambaInstructLLM {
    /// Creates a new `Ai21JambaInstructLLM` instance.
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

impl LLMProvider for Ai21JambaInstructLLM {
    fn provider_name(&self) -> &'static str {
        "ai21_jamba_instruct"
    }
}

impl ProviderConfig for Ai21JambaInstructLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
