//! PromptLayer callback handler.

use chrono::Utc;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::traits::CallbackHandler;

pub struct PromptlayerHandler {
    api_key: String,
    base_url: String,
    sequence: AtomicU64,
}

impl Default for PromptlayerHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptlayerHandler {
    pub fn new() -> Self {
        let api_key = std::env::var("PROMPTLAYER_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://api.promptlayer.com".into(),
            sequence: AtomicU64::new(0),
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = key.to_string();
        self
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    #[allow(dead_code)]
    async fn log_prompt(&self, function_name: &str, data: &Value) {
        if self.api_key.is_empty() {
            tracing::warn!("PROMPTLAYER_API_KEY not set, skipping log");
            return;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/track", self.base_url);

        let payload = serde_json::json!({
            "api_key": self.api_key,
            "function_name": function_name,
            "data": data,
            "timestamp": Utc::now().to_rfc3339(),
        });

        match client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    tracing::warn!(
                        "Promptlayer log failed: {} {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Promptlayer log error: {}", e);
            }
        }
    }
}

impl CallbackHandler for PromptlayerHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        tracing::debug!(
            "Promptlayer[{}] chain start: {} inputs={}",
            seq,
            name,
            inputs
        );
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        tracing::debug!("Promptlayer chain end: {} outputs={}", name, outputs);
    }

    fn on_llm_start(&self, name: &str, prompts: &[String]) {
        tracing::debug!(
            "Promptlayer llm start: {} {} prompt(s)",
            name,
            prompts.len()
        );
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        tracing::debug!("Promptlayer llm end: {} output={}", name, output);
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        tracing::debug!("Promptlayer tool start: {} input={}", name, input);
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        tracing::debug!("Promptlayer tool end: {} output={}", name, output);
    }

    fn on_agent_action(&self, action: &Value) {
        tracing::debug!("Promptlayer agent action: {}", action);
    }

    fn on_agent_finish(&self, finish: &Value) {
        tracing::debug!("Promptlayer agent finish: {}", finish);
    }
}
