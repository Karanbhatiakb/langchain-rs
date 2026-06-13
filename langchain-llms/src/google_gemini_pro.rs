//! Google Gemini Pro LLM provider — Gemini Pro models via Gemini API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1";

/// LLM provider for Google Gemini Pro models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGeminiProLLM {
    /// Google API key.
    pub api_key: String,
    /// Gemini Pro model identifier (e.g. "gemini-1.5-pro").
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Gemini API base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl GoogleGeminiProLLM {
    /// Creates a new `GoogleGeminiProLLM` with the given model and API key.
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
        warn!("GoogleGeminiProLLM is a stub — no real API call made");
        format!("[GoogleGeminiProLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for GoogleGeminiProLLM {
    fn provider_name(&self) -> &'static str {
        "google_gemini_pro"
    }
}

impl ProviderConfig for GoogleGeminiProLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
