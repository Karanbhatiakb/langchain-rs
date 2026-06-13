//! Cohere Command LLM provider — Command models via Cohere API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.cohere.com/v1";

/// LLM provider for Cohere Command models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereCommandLLM {
    /// Cohere API key.
    pub api_key: String,
    /// Command model identifier (e.g. "command").
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Cohere API base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl CohereCommandLLM {
    /// Creates a new `CohereCommandLLM` with the given model and API key.
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
        warn!("CohereCommandLLM is a stub — no real API call made");
        format!("[CohereCommandLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for CohereCommandLLM {
    fn provider_name(&self) -> &'static str {
        "cohere_command"
    }
}

impl ProviderConfig for CohereCommandLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
