//! ClearML callback handler.

use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

use crate::traits::CallbackHandler;

pub struct ClearMLTracer {
    project_name: String,
    task_name: Option<String>,
    sequence: AtomicU64,
}

impl Default for ClearMLTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl ClearMLTracer {
    pub fn new() -> Self {
        let project_name =
            std::env::var("CLEARML_PROJECT").unwrap_or_else(|_| "langchain".into());
        Self {
            project_name,
            task_name: None,
            sequence: AtomicU64::new(0),
        }
    }

    pub fn with_project(mut self, name: &str) -> Self {
        self.project_name = name.to_string();
        self
    }

    pub fn with_task_name(mut self, name: &str) -> Self {
        self.task_name = Some(name.to_string());
        self
    }
}

impl CallbackHandler for ClearMLTracer {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        info!(
            "[ClearML] project={} task={:?} chain[{}]/{} started: {}",
            self.project_name,
            self.task_name,
            seq,
            name,
            inputs
        );
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        info!(
            "[ClearML] project={} chain/{} ended: {}",
            self.project_name, name, outputs
        );
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        info!(
            "[ClearML] project={} llm/{} output: {}",
            self.project_name, name, output
        );
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        info!(
            "[ClearML] project={} tool/{} output: {}",
            self.project_name, name, output
        );
    }

    fn on_agent_finish(&self, finish: &Value) {
        info!(
            "[ClearML] project={} agent finished: {}",
            self.project_name, finish
        );
    }
}
