//! GPT4All v2 LLM provider — local model inference via GPT4All.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "http://localhost:4891";

/// LLM provider for GPT4All v2 local models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gpt4AllV2LLM {
    /// API key (may be empty for local-only setups).
    pub api_key: String,
    /// Model identifier.
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// GPT4All API base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl Gpt4AllV2LLM {
    /// Creates a new `Gpt4AllV2LLM` with the given model and API key.
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
        warn!("Gpt4AllV2LLM is a stub — no real API call made");
        format!("[Gpt4AllV2LLM stub] received: {}", prompt)
    }
}

impl LLMProvider for Gpt4AllV2LLM {
    fn provider_name(&self) -> &'static str {
        "gpt4all_v2"
    }
}

impl ProviderConfig for Gpt4AllV2LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
