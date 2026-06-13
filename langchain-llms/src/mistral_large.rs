//! Mistral Large LLM provider — Mistral Large models via Mistral API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.mistral.ai/v1";

/// LLM provider for Mistral Large models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralLargeLLM {
    /// Mistral API key.
    pub api_key: String,
    /// Mistral Large model identifier (e.g. "mistral-large-latest").
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Mistral API base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl MistralLargeLLM {
    /// Creates a new `MistralLargeLLM` with the given model and API key.
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
        warn!("MistralLargeLLM is a stub — no real API call made");
        format!("[MistralLargeLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for MistralLargeLLM {
    fn provider_name(&self) -> &'static str {
        "mistral_large"
    }
}

impl ProviderConfig for MistralLargeLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
