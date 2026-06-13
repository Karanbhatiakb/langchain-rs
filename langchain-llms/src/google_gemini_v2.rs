//! Google Gemini v2 LLM provider — Gemini 2.0 models via Gemini API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1";

/// LLM provider for Google Gemini 2.0 models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGeminiV2LLM {
    /// Google API key.
    pub api_key: String,
    /// Gemini 2.0 model identifier (e.g. "gemini-2.0-flash").
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

impl GoogleGeminiV2LLM {
    /// Creates a new `GoogleGeminiV2LLM` with the given model and API key.
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
        warn!("GoogleGeminiV2LLM is a stub — no real API call made");
        format!("[GoogleGeminiV2LLM stub] received: {}", prompt)
    }
}

impl LLMProvider for GoogleGeminiV2LLM {
    fn provider_name(&self) -> &'static str {
        "google_gemini_v2"
    }
}

impl ProviderConfig for GoogleGeminiV2LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
