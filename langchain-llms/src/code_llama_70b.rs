//! CodeLlama70bLLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "code_llama_70b")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// CodeLlama70bLLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLlama70bLLM {
    /// API key for authentication.
    pub api_key: String,
    /// Model identifier.
    pub model: String,
    /// Sampling temperature.
    pub temperature: Option<f64>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Base URL for API requests.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
    /// Provider-specific configuration.
    pub provider_specific: Value,
}

impl CodeLlama70bLLM {
    /// Creates a new `CodeLlama70bLLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://api.codellama.ai/v1".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for CodeLlama70bLLM {
    fn provider_name(&self) -> &'static str {
        "code_llama_70b"
    }
}

impl ProviderConfig for CodeLlama70bLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
