//! Weights & Biases callback handler.

use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

use crate::traits::CallbackHandler;

pub struct WandbTracer {
    project: String,
    run_name: Option<String>,
    #[allow(dead_code)]
    config: HashMap<String, Value>,
    logged_metrics: Vec<HashMap<String, Value>>,
}

impl Default for WandbTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl WandbTracer {
    pub fn new() -> Self {
        let project = std::env::var("WANDB_PROJECT").unwrap_or_else(|_| "langchain".into());
        Self {
            project,
            run_name: None,
            config: HashMap::new(),
            logged_metrics: Vec::new(),
        }
    }

    pub fn with_project(mut self, project: &str) -> Self {
        self.project = project.to_string();
        self
    }

    pub fn with_run_name(mut self, name: &str) -> Self {
        self.run_name = Some(name.to_string());
        self
    }

    pub fn log_metric(&mut self, key: &str, value: Value) {
        info!("[W&B] {}: {} (project: {})", key, value, self.project);
        let mut metric = HashMap::new();
        metric.insert(key.to_string(), value);
        self.logged_metrics.push(metric);
    }
}

impl CallbackHandler for WandbTracer {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        info!("[W&B] chain/{} started: {}", name, inputs);
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        info!("[W&B] chain/{} ended: {}", name, outputs);
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        info!("[W&B] llm/{} output: {}", name, output);
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        info!("[W&B] tool/{} output: {}", name, output);
    }

    fn on_agent_finish(&self, finish: &Value) {
        info!("[W&B] agent finished: {}", finish);
    }
}
