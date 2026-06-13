//! Alibaba Qwen LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "alibaba_qwen")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Alibaba Qwen LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlibabaQwenLLM {
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

impl AlibabaQwenLLM {
    /// Creates a new `AlibabaQwenLLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://dashscope.aliyuncs.com/api/v1".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for AlibabaQwenLLM {
    fn provider_name(&self) -> &'static str {
        "alibaba_qwen"
    }
}

impl ProviderConfig for AlibabaQwenLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
