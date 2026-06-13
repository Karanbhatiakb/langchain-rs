//! Anthropic Claude 4 LLM provider — Claude 4 models via Anthropic API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.anthropic.com/v1";

/// LLM provider for Anthropic Claude 4 models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicClaudeV4LLM {
    /// Anthropic API key.
    pub api_key: String,
    /// Claude 4 model identifier (e.g. "claude-4-opus-20250514").
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Anthropic API base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl AnthropicClaudeV4LLM {
    /// Creates a new `AnthropicClaudeV4LLM` with the given model and API key.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            temperature: 0.7,
            max_tokens: 1024,
            base_url: BASE_URL.to_string(),
            config: GenerationConfig::default(),
        }
    }

    /// Stub generate — logs a warning and returns a placeholder response.
    pub fn generate(&self, prompt: &str) -> String {
        warn!("AnthropicClaudeV4LLM is a stub — no real API call made");
        format!("[AnthropicClaudeV4LLM stub] received: {}", prompt)
    }
}

impl LLMProvider for AnthropicClaudeV4LLM {
    fn provider_name(&self) -> &'static str {
        "anthropic_claude_v4"
    }
}

impl ProviderConfig for AnthropicClaudeV4LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
