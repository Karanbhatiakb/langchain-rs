//! Cohere Command R LLM provider — Command R models via Cohere API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.cohere.com/v1";

/// LLM provider for Cohere Command R / R+ models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereCommandRLLM {
    /// Cohere API key.
    pub api_key: String,
    /// Command R model identifier (e.g. "command-r").
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

impl CohereCommandRLLM {
    /// Creates a new `CohereCommandRLLM` with the given model and API key.
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
        warn!("CohereCommandRLLM is a stub — no real API call made");
        format!("[CohereCommandRLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for CohereCommandRLLM {
    fn provider_name(&self) -> &'static str {
        "cohere_command_r"
    }
}

impl ProviderConfig for CohereCommandRLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
