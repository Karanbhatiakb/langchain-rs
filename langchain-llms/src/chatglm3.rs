//! Chatglm3LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "chatglm3")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Chatglm3LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chatglm3LLM {
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

impl Chatglm3LLM {
    /// Creates a new `Chatglm3LLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for Chatglm3LLM {
    fn provider_name(&self) -> &'static str {
        "chatglm3"
    }
}

impl ProviderConfig for Chatglm3LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
