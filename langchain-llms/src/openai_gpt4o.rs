//! OpenAI GPT-4o LLM provider — GPT-4o models via OpenAI API.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.openai.com/v1";

/// LLM provider for OpenAI GPT-4o models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiGpt4OLLM {
    /// OpenAI API key.
    pub api_key: String,
    /// GPT-4o model identifier (e.g. "gpt-4o").
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

impl OpenAiGpt4OLLM {
    /// Creates a new `OpenAiGpt4OLLM` with the given model and API key.
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
        warn!("OpenAiGpt4OLLM is a stub — no real API call made");
        format!("[OpenAiGpt4OLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for OpenAiGpt4OLLM {
    fn provider_name(&self) -> &'static str {
        "openai_gpt4o"
    }
}

impl ProviderConfig for OpenAiGpt4OLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
