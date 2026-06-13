//! Groq Mixtral LLM provider — Mixtral models on GroqCloud.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.groq.com/openai/v1";

/// LLM provider for Mixtral models served on GroqCloud.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqMixtralLLM {
    /// GroqCloud API key.
    pub api_key: String,
    /// Mixtral model identifier (e.g. "mixtral-8x7b-32768").
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Groq API base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl GroqMixtralLLM {
    /// Creates a new `GroqMixtralLLM` with the given model and API key.
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
        warn!("GroqMixtralLLM is a stub — no real API call made");
        format!("[GroqMixtralLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for GroqMixtralLLM {
    fn provider_name(&self) -> &'static str {
        "groq_mixtral"
    }
}

impl ProviderConfig for GroqMixtralLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
