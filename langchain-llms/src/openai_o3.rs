//! OpenAI o3 LLM provider — o3 reasoning models via OpenAI API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.openai.com/v1";

/// LLM provider for OpenAI o3 reasoning models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiO3LLM {
    /// OpenAI API key.
    pub api_key: String,
    /// o3 model identifier (e.g. "o3-mini").
    pub model: String,
    /// Sampling temperature (0.0–2.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// OpenAI API base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl OpenAiO3LLM {
    /// Creates a new `OpenAiO3LLM` with the given model and API key.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            temperature: 0.7,
            max_tokens: 4096,
            base_url: BASE_URL.to_string(),
            config: GenerationConfig::default(),
        }
    }

    /// Stub generate — logs a warning and returns a placeholder response.
    pub fn generate(&self, prompt: &str) -> String {
        warn!("OpenAiO3LLM is a stub — no real API call made");
        format!("[OpenAiO3LLM stub] received: {}", prompt)
    }
}

impl LLMProvider for OpenAiO3LLM {
    fn provider_name(&self) -> &'static str {
        "openai_o3"
    }
}

impl ProviderConfig for OpenAiO3LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
