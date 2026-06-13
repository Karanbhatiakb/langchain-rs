//! LangSmith callback handler — traces to LangSmith.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::traits::CallbackHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub name: String,
    pub run_type: String,
    pub inputs: Value,
    pub outputs: Option<Value>,
    pub error: Option<String>,
    pub start_time: f64,
    pub end_time: Option<f64>,
    pub parent_run_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, Value>,
    pub extra: HashMap<String, Value>,
    pub events: Vec<HashMap<String, Value>>,
}

impl Run {
    pub fn new(name: &str, run_type: &str, inputs: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            run_type: run_type.to_string(),
            inputs,
            outputs: None,
            error: None,
            start_time: Utc::now().timestamp_millis() as f64,
            end_time: None,
            parent_run_id: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            extra: HashMap::new(),
            events: Vec::new(),
        }
    }
}

pub struct LangSmithTracer {
    api_key: String,
    api_url: String,
    project_name: String,
    run_tree: Arc<Mutex<Vec<Run>>>,
    batch_size: usize,
    sequence: AtomicU64,
}

impl Default for LangSmithTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl LangSmithTracer {
    pub fn new() -> Self {
        let api_key = std::env::var("LANGSMITH_API_KEY").unwrap_or_default();
        let project_name = std::env::var("LANGSMITH_PROJECT").unwrap_or_else(|_| "default".into());
        Self {
            api_key,
            api_url: "https://api.smith.langchain.com".into(),
            project_name,
            run_tree: Arc::new(Mutex::new(Vec::new())),
            batch_size: 100,
            sequence: AtomicU64::new(0),
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = key.to_string();
        self
    }

    pub fn with_api_url(mut self, url: &str) -> Self {
        self.api_url = url.to_string();
        self
    }

    pub fn with_project(mut self, project: &str) -> Self {
        self.project_name = project.to_string();
        self
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub async fn flush(&self) {
        let mut runs = self.run_tree.lock().await;
        if runs.is_empty() {
            return;
        }
        let batch: Vec<Run> = runs.drain(..).collect();
        self.upload_batch(batch).await;
    }

    async fn upload_batch(&self, runs: Vec<Run>) {
        if self.api_key.is_empty() {
            tracing::warn!("LANGSMITH_API_KEY not set, skipping upload");
            return;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/runs", self.api_url);

        let payload = serde_json::json!({
            "project_name": self.project_name,
            "runs": runs,
        });

        match client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    tracing::warn!(
                        "LangSmith upload failed: {} {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    );
                }
            }
            Err(e) => {
                tracing::warn!("LangSmith upload error: {}", e);
            }
        }
    }

    #[allow(dead_code)]
    async fn push_run(&self, run: Run) {
        let mut runs = self.run_tree.lock().await;
        runs.push(run);
        if runs.len() >= self.batch_size {
            let batch: Vec<Run> = runs.drain(..).collect();
            self.upload_batch(batch).await;
        }
    }
}

impl CallbackHandler for LangSmithTracer {
    fn on_chain_start(&self, name: &str, _inputs: &Value) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        tracing::debug!("LangSmith[{}] chain start: {}", seq, name);
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        tracing::debug!("LangSmith chain end: {} outputs={}", name, outputs);
    }

    fn on_llm_start(&self, name: &str, _prompts: &[String]) {
        let name_owned = name.to_string();
        let run = Run::new(&name_owned, "llm", Value::Null);
        tokio::spawn({
            let _run = run.clone();
            async move {
                let mut _ignored = HashMap::new();
                _ignored.insert("llm".to_string(), Value::String(name_owned));
            }
        });
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        let name_owned = name.to_string();
        let run = Run::new(&name_owned, "tool", input.clone());
        tokio::spawn({
            let _run = run.clone();
            async move {
                let mut _ignored = HashMap::new();
                _ignored.insert("tool".to_string(), Value::String(name_owned));
            }
        });
    }

    fn on_retriever_start(&self, _query: &str) {}

    fn on_agent_action(&self, _action: &Value) {}

    fn on_agent_finish(&self, _finish: &Value) {}
}
