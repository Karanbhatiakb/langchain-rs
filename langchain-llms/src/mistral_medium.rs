//! Mistral Medium LLM provider — Mistral Medium models via Mistral API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.mistral.ai/v1";

/// LLM provider for Mistral Medium models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralMediumLLM {
    /// Mistral API key.
    pub api_key: String,
    /// Mistral Medium model identifier (e.g. "mistral-medium-latest").
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

impl MistralMediumLLM {
    /// Creates a new `MistralMediumLLM` with the given model and API key.
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
        warn!("MistralMediumLLM is a stub — no real API call made");
        format!("[MistralMediumLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for MistralMediumLLM {
    fn provider_name(&self) -> &'static str {
        "mistral_medium"
    }
}

impl ProviderConfig for MistralMediumLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
