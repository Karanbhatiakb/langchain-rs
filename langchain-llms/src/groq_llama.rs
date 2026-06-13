//! Groq Llama LLM provider — Meta Llama models on GroqCloud.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://api.groq.com/openai/v1";

/// LLM provider for Meta Llama models served on GroqCloud.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqLlamaLLM {
    /// GroqCloud API key.
    pub api_key: String,
    /// Llama model identifier (e.g. "llama3-70b-8192").
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

impl GroqLlamaLLM {
    /// Creates a new `GroqLlamaLLM` with the given model and API key.
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
        warn!("GroqLlamaLLM is a stub — no real API call made");
        format!("[GroqLlamaLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for GroqLlamaLLM {
    fn provider_name(&self) -> &'static str {
        "groq_llama"
    }
}

impl ProviderConfig for GroqLlamaLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
