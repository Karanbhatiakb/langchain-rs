//! Chatglm4LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "chatglm4")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Chatglm4LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chatglm4LLM {
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

impl Chatglm4LLM {
    /// Creates a new `Chatglm4LLM` instance.
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

impl LLMProvider for Chatglm4LLM {
    fn provider_name(&self) -> &'static str {
        "chatglm4"
    }
}

impl ProviderConfig for Chatglm4LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
