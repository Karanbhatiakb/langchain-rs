//! Anthropic Claude 3 Opus LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "anthropic_claude_3_opus")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Anthropic Claude 3 Opus LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicClaude3OpusLLM {
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

impl AnthropicClaude3OpusLLM {
    /// Creates a new `AnthropicClaude3OpusLLM` instance.
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

impl LLMProvider for AnthropicClaude3OpusLLM {
    fn provider_name(&self) -> &'static str {
        "anthropic_claude_3_opus"
    }
}

impl ProviderConfig for AnthropicClaude3OpusLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
