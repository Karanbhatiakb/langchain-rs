//! LlamaCpp v2 LLM provider — local inference via llama.cpp server.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "http://localhost:8080";

/// LLM provider for llama.cpp v2 server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaCppV2LLM {
    /// API key (may be empty for local setups).
    pub api_key: String,
    /// Model identifier (e.g. "llama-2-7b").
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// llama.cpp server base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl LlamaCppV2LLM {
    /// Creates a new `LlamaCppV2LLM` with the given model and API key.
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
        warn!("LlamaCppV2LLM is a stub — no real API call made");
        format!("[LlamaCppV2LLM stub] received: {}", prompt)
    }
}

impl LLMProvider for LlamaCppV2LLM {
    fn provider_name(&self) -> &'static str {
        "llamacpp_v2"
    }
}

impl ProviderConfig for LlamaCppV2LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
