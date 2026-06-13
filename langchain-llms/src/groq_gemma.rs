//! Groq Gemma LLM provider — Google Gemma models on GroqCloud.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.groq.com/openai/v1";

/// LLM provider for Google Gemma models served on GroqCloud.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqGemmaLLM {
    /// GroqCloud API key.
    pub api_key: String,
    /// Gemma model identifier (e.g. "gemma2-9b-it").
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

impl GroqGemmaLLM {
    /// Creates a new `GroqGemmaLLM` with the given model and API key.
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
        warn!("GroqGemmaLLM is a stub — no real API call made");
        format!("[GroqGemmaLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for GroqGemmaLLM {
    fn provider_name(&self) -> &'static str {
        "groq_gemma"
    }
}

impl ProviderConfig for GroqGemmaLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
