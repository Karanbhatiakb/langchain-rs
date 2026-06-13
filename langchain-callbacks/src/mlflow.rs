//! MLflow callback handler.

use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

use crate::traits::CallbackHandler;

pub struct MLflowTracer {
    experiment_name: String,
    run_name: Option<String>,
    tracking_uri: String,
    sequence: AtomicU64,
}

impl Default for MLflowTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl MLflowTracer {
    pub fn new() -> Self {
        let tracking_uri = std::env::var("MLFLOW_TRACKING_URI")
            .unwrap_or_else(|_| "http://localhost:5000".into());
        let experiment_name =
            std::env::var("MLFLOW_EXPERIMENT_NAME").unwrap_or_else(|_| "langchain".into());
        Self {
            experiment_name,
            run_name: None,
            tracking_uri,
            sequence: AtomicU64::new(0),
        }
    }

    pub fn with_tracking_uri(mut self, uri: &str) -> Self {
        self.tracking_uri = uri.to_string();
        self
    }

    pub fn with_experiment(mut self, name: &str) -> Self {
        self.experiment_name = name.to_string();
        self
    }

    pub fn with_run_name(mut self, name: &str) -> Self {
        self.run_name = Some(name.to_string());
        self
    }
}

impl CallbackHandler for MLflowTracer {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        info!(
            "[MLflow] experiment={} run={} chain[{}]/{} started: {}",
            self.experiment_name,
            self.run_name.as_deref().unwrap_or("default"),
            seq,
            name,
            inputs
        );
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        info!(
            "[MLflow] experiment={} chain/{}, ended: {}",
            self.experiment_name, name, outputs
        );
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        info!(
            "[MLflow] experiment={} llm/{} output: {}",
            self.experiment_name, name, output
        );
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        info!(
            "[MLflow] experiment={} tool/{} output: {}",
            self.experiment_name, name, output
        );
    }

    fn on_agent_action(&self, action: &Value) {
        info!(
            "[MLflow] experiment={} agent action: {}",
            self.experiment_name, action
        );
    }
}
