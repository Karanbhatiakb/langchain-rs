//! Helicone callback handler.

use chrono::Utc;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::traits::CallbackHandler;

pub struct HeliconeHandler {
    api_key: String,
    base_url: String,
    sequence: AtomicU64,
}

impl Default for HeliconeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl HeliconeHandler {
    pub fn new() -> Self {
        let api_key = std::env::var("HELICONE_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://api.helicone.ai/v1".into(),
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
    async fn log_request(&self, request_type: &str, data: &Value) {
        if self.api_key.is_empty() {
            tracing::warn!("HELICONE_API_KEY not set, skipping log");
            return;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/request", self.base_url);

        let payload = serde_json::json!({
            "request_type": request_type,
            "data": data,
            "timestamp": Utc::now().to_rfc3339(),
        });

        match client
            .post(&url)
            .header("Helicone-Auth", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    tracing::warn!(
                        "Helicone log failed: {} {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Helicone log error: {}", e);
            }
        }
    }
}

impl CallbackHandler for HeliconeHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        tracing::debug!(
            "Helicone[{}] chain start: {} inputs={}",
            seq,
            name,
            inputs
        );
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        tracing::debug!("Helicone chain end: {} outputs={}", name, outputs);
    }

    fn on_chain_error(&self, name: &str, error: &Value) {
        tracing::debug!("Helicone chain error: {} error={}", name, error);
    }

    fn on_llm_start(&self, name: &str, prompts: &[String]) {
        tracing::debug!(
            "Helicone llm start: {} {} prompt(s)",
            name,
            prompts.len()
        );
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        tracing::debug!("Helicone llm end: {} output={}", name, output);
    }

    fn on_llm_error(&self, name: &str, error: &Value) {
        tracing::debug!("Helicone llm error: {} error={}", name, error);
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        tracing::debug!("Helicone tool start: {} input={}", name, input);
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        tracing::debug!("Helicone tool end: {} output={}", name, output);
    }

    fn on_tool_error(&self, name: &str, error: &Value) {
        tracing::debug!("Helicone tool error: {} error={}", name, error);
    }

    fn on_agent_action(&self, action: &Value) {
        tracing::debug!("Helicone agent action: {}", action);
    }

    fn on_agent_finish(&self, finish: &Value) {
        tracing::debug!("Helicone agent finish: {}", finish);
    }
}
