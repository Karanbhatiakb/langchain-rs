//! Bedrock Stable Diffusion LLM provider — SDXL image generation via AWS Bedrock.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::types::{GenerationConfig, LLMProvider, ProviderConfig};

const BASE_URL: &str = "https://bedrock-runtime.us-east-1.amazonaws.com";

/// LLM provider for Stable Diffusion XL models served through AWS Bedrock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockStableDiffusionLLM {
    /// AWS access key or IAM role credential.
    pub api_key: String,
    /// SDXL model identifier (e.g. "stability.stable-diffusion-xl-v1").
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

impl BedrockStableDiffusionLLM {
    /// Creates a new `BedrockStableDiffusionLLM` with the given model and API key.
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
        warn!("BedrockStableDiffusionLLM is a stub — no real API call made");
        format!("[BedrockStableDiffusionLLM stub] received: {}", prompt)
    }
}

impl LLMProvider for BedrockStableDiffusionLLM {
    fn provider_name(&self) -> &'static str {
        "bedrock_stable_diffusion"
    }
}

impl ProviderConfig for BedrockStableDiffusionLLM {
    fn config(&self) -> &GenerationConfig {
        &self.config
    }
}
