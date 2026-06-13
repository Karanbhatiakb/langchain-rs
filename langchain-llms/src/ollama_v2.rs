//! Ollama v2 LLM provider — local model inference via Ollama.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "http://localhost:11434";

/// LLM provider for Ollama v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaV2LLM {
    /// API key (may be empty for local setups).
    pub api_key: String,
    /// Model identifier (e.g. "llama3.1").
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Ollama API base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl OllamaV2LLM {
    /// Creates a new `OllamaV2LLM` with the given model and API key.
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
        warn!("OllamaV2LLM is a stub — no real API call made");
        format!("[OllamaV2LLM stub] received: {}", prompt)
    }
}

impl LLMProvider for OllamaV2LLM {
    fn provider_name(&self) -> &'static str {
        "ollama_v2"
    }
}

impl ProviderConfig for OllamaV2LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
