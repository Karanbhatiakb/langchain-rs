//! Portkey callback handler.

use chrono::Utc;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::traits::CallbackHandler;

pub struct PortkeyHandler {
    api_key: String,
    base_url: String,
    config_id: Option<String>,
    sequence: AtomicU64,
}

impl Default for PortkeyHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl PortkeyHandler {
    pub fn new() -> Self {
        let api_key = std::env::var("PORTKEY_API_KEY").unwrap_or_default();
        let config_id = std::env::var("PORTKEY_CONFIG_ID").ok();
        Self {
            api_key,
            base_url: "https://api.portkey.ai/v1".into(),
            config_id,
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

    pub fn with_config_id(mut self, id: &str) -> Self {
        self.config_id = Some(id.to_string());
        self
    }

    #[allow(dead_code)]
    async fn log_event(&self, event_type: &str, name: &str, data: &Value) {
        if self.api_key.is_empty() {
            tracing::warn!("PORTKEY_API_KEY not set, skipping log");
            return;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/logs", self.base_url);

        let mut payload = serde_json::json!({
            "event_type": event_type,
            "name": name,
            "data": data,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Some(ref config_id) = self.config_id {
            payload["config_id"] = Value::String(config_id.clone());
        }

        match client
            .post(&url)
            .header("x-portkey-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    tracing::warn!(
                        "Portkey log failed: {} {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Portkey log error: {}", e);
            }
        }
    }
}

impl CallbackHandler for PortkeyHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        tracing::debug!("Portkey[{}] chain start: {} inputs={}", seq, name, inputs);
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        tracing::debug!("Portkey chain end: {} outputs={}", name, outputs);
    }

    fn on_llm_start(&self, name: &str, prompts: &[String]) {
        tracing::debug!("Portkey llm start: {} {} prompt(s)", name, prompts.len());
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        tracing::debug!("Portkey llm end: {} output={}", name, output);
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        tracing::debug!("Portkey tool start: {} input={}", name, input);
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        tracing::debug!("Portkey tool end: {} output={}", name, output);
    }

    fn on_agent_action(&self, action: &Value) {
        tracing::debug!("Portkey agent action: {}", action);
    }

    fn on_agent_finish(&self, finish: &Value) {
        tracing::debug!("Portkey agent finish: {}", finish);
    }
}
