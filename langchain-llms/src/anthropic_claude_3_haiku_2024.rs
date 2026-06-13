//! AnthropicClaude3Haiku2024LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "anthropic_claude_3_haiku_2024")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// AnthropicClaude3Haiku2024LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicClaude3Haiku2024LLM {
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

impl AnthropicClaude3Haiku2024LLM {
    /// Creates a new `AnthropicClaude3Haiku2024LLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://api.anthropic.com/v1".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for AnthropicClaude3Haiku2024LLM {
    fn provider_name(&self) -> &'static str {
        "anthropic_claude_3_haiku_2024"
    }
}

impl ProviderConfig for AnthropicClaude3Haiku2024LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
