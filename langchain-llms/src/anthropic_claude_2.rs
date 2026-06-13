//! Anthropic Claude 2 LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "anthropic_claude_2")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Anthropic Claude 2 LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicClaude2LLM {
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

impl AnthropicClaude2LLM {
    /// Creates a new `AnthropicClaude2LLM` instance.
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

impl LLMProvider for AnthropicClaude2LLM {
    fn provider_name(&self) -> &'static str {
        "anthropic_claude_2"
    }
}

impl ProviderConfig for AnthropicClaude2LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
