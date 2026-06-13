//! Comet ML callback handler.

use chrono::Utc;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::traits::CallbackHandler;

pub struct CometHandler {
    api_key: String,
    #[allow(dead_code)]
    base_url: String,
    project_name: String,
    workspace: Option<String>,
    experiment_key: Option<String>,
    sequence: AtomicU64,
}

impl Default for CometHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CometHandler {
    pub fn new() -> Self {
        let api_key = std::env::var("COMET_API_KEY").unwrap_or_default();
        let project_name = std::env::var("COMET_PROJECT_NAME")
            .unwrap_or_else(|_| "langchain-rs".into());
        let workspace = std::env::var("COMET_WORKSPACE").ok();
        Self {
            api_key,
            base_url: "https://www.comet.com/clientlib".into(),
            project_name,
            workspace,
            experiment_key: None,
            sequence: AtomicU64::new(0),
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = key.to_string();
        self
    }

    pub fn with_project(mut self, name: &str) -> Self {
        self.project_name = name.to_string();
        self
    }

    pub fn with_workspace(mut self, workspace: &str) -> Self {
        self.workspace = Some(workspace.to_string());
        self
    }

    pub fn with_experiment_key(mut self, key: &str) -> Self {
        self.experiment_key = Some(key.to_string());
        self
    }

    #[allow(dead_code)]
    async fn log_metric(&self, metric_name: &str, value: f64, step: u64) {
        if self.api_key.is_empty() {
            tracing::warn!("COMET_API_KEY not set, skipping log");
            return;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/api/rest/v2/write/metric", self.base_url);

        let mut payload = serde_json::json!({
            "apiKey": self.api_key,
            "projectName": self.project_name,
            "metricName": metric_name,
            "metricValue": value,
            "step": step,
            "timestamp": Utc::now().timestamp_millis(),
        });

        if let Some(ref ws) = self.workspace {
            payload["workspace"] = Value::String(ws.clone());
        }
        if let Some(ref key) = self.experiment_key {
            payload["experimentKey"] = Value::String(key.clone());
        }

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
                        "Comet log failed: {} {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Comet log error: {}", e);
            }
        }
    }
}

impl CallbackHandler for CometHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        tracing::debug!(
            "Comet[{}] experiment={} chain start: {} inputs={}",
            seq,
            self.project_name,
            name,
            inputs
        );
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        tracing::debug!(
            "Comet experiment={} chain end: {} outputs={}",
            self.project_name,
            name,
            outputs
        );
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        tracing::debug!(
            "Comet experiment={} llm/{} output: {}",
            self.project_name,
            name,
            output
        );
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        tracing::debug!(
            "Comet experiment={} tool/{} output: {}",
            self.project_name,
            name,
            output
        );
    }

    fn on_agent_action(&self, action: &Value) {
        tracing::debug!(
            "Comet experiment={} agent action: {}",
            self.project_name,
            action
        );
    }

    fn on_agent_finish(&self, finish: &Value) {
        tracing::debug!(
            "Comet experiment={} agent finish: {}",
            self.project_name,
            finish
        );
    }
}
