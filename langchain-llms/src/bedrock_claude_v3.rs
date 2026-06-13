//! Bedrock Claude v3 LLM provider — Anthropic Claude 3 models via AWS Bedrock.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://bedrock-runtime.us-east-1.amazonaws.com";

/// LLM provider for Anthropic Claude 3 models served through AWS Bedrock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockClaudeV3LLM {
    /// AWS access key or IAM role credential.
    pub api_key: String,
    /// Claude 3 model identifier (e.g. "anthropic.claude-3-sonnet-20240229").
    pub model: String,
    /// Sampling temperature (0.0–1.0).
    pub temperature: f64,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Bedrock runtime base URL.
    pub base_url: String,
    /// Generation configuration.
    pub config: GenerationConfig,
}

impl BedrockClaudeV3LLM {
    /// Creates a new `BedrockClaudeV3LLM` with the given model and API key.
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
        warn!("BedrockClaudeV3LLM is a stub — no real API call made");
        format!("[BedrockClaudeV3LLM stub] received: {}", prompt)
    }
}

impl LLMProvider for BedrockClaudeV3LLM {
    fn provider_name(&self) -> &'static str {
        "bedrock_claude_v3"
    }
}

impl ProviderConfig for BedrockClaudeV3LLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
