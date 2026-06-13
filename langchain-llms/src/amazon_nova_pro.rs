//! Amazon Nova Pro LLM provider.
//!
//! ## Feature flag
//! `#[cfg(feature = "amazon_nova_pro")]`

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

/// Amazon Nova Pro LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmazonNovaProLLM {
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

impl AmazonNovaProLLM {
    /// Creates a new `AmazonNovaProLLM` instance.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            temperature: None,
            max_tokens: None,
            base_url: "https://bedrock-runtime.us-east-1.amazonaws.com".to_string(),
            config: GenerationConfig::default(),
            provider_specific: Value::Null,
        }
    }
}

impl LLMProvider for AmazonNovaProLLM {
    fn provider_name(&self) -> &'static str {
        "amazon_nova_pro"
    }
}

impl ProviderConfig for AmazonNovaProLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
