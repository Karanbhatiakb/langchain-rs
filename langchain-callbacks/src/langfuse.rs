//! Langfuse callback handler.

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
pub struct LangFuseSpan {
    pub id: Uuid,
    pub trace_id: Uuid,
    pub name: String,
    pub span_type: String,
    pub inputs: Value,
    pub outputs: Option<Value>,
    pub error: Option<String>,
    pub start_time: f64,
    pub end_time: Option<f64>,
    pub metadata: HashMap<String, Value>,
}

pub struct LangFuseHandler {
    api_key: String,
    api_url: String,
    project_name: String,
    spans: Arc<Mutex<Vec<LangFuseSpan>>>,
    batch_size: usize,
    sequence: AtomicU64,
}

impl Default for LangFuseHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LangFuseHandler {
    pub fn new() -> Self {
        let api_key = std::env::var("LANGFUSE_API_KEY").unwrap_or_default();
        let project_name = std::env::var("LANGFUSE_PROJECT").unwrap_or_else(|_| "default".into());
        Self {
            api_key,
            api_url: "https://cloud.langfuse.com/api/public".into(),
            project_name,
            spans: Arc::new(Mutex::new(Vec::new())),
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
        let mut spans = self.spans.lock().await;
        if spans.is_empty() {
            return;
        }
        let batch: Vec<LangFuseSpan> = spans.drain(..).collect();
        self.upload_batch(batch).await;
    }

    async fn upload_batch(&self, spans: Vec<LangFuseSpan>) {
        if self.api_key.is_empty() {
            tracing::warn!("LANGFUSE_API_KEY not set, skipping upload");
            return;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/ingestion", self.api_url);

        let payload = serde_json::json!({
            "batch": spans,
            "metadata": {
                "project_name": self.project_name,
            },
        });

        match client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    tracing::warn!(
                        "LangFuse upload failed: {} {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    );
                }
            }
            Err(e) => {
                tracing::warn!("LangFuse upload error: {}", e);
            }
        }
    }

    fn create_span(&self, name: &str, span_type: &str, inputs: Value) -> LangFuseSpan {
        LangFuseSpan {
            id: Uuid::new_v4(),
            trace_id: Uuid::new_v4(),
            name: name.to_string(),
            span_type: span_type.to_string(),
            inputs,
            outputs: None,
            error: None,
            start_time: Utc::now().timestamp_millis() as f64,
            end_time: None,
            metadata: HashMap::new(),
        }
    }
}

impl CallbackHandler for LangFuseHandler {
    fn on_chain_start(&self, name: &str, _inputs: &Value) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        tracing::debug!("LangFuse[{}] chain start: {}", seq, name);
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        tracing::debug!("LangFuse chain end: {} outputs={}", name, outputs);
    }

    fn on_chain_error(&self, name: &str, error: &Value) {
        tracing::debug!("LangFuse chain error: {} error={}", name, error);
    }

    fn on_llm_start(&self, name: &str, _prompts: &[String]) {
        let _span = self.create_span(name, "llm", Value::Null);
        tracing::debug!("LangFuse llm start: {}", name);
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        tracing::debug!("LangFuse llm end: {} output={}", name, output);
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        let _span = self.create_span(name, "tool", input.clone());
        tracing::debug!("LangFuse tool start: {}", name);
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        tracing::debug!("LangFuse tool end: {} output={}", name, output);
    }

    fn on_agent_action(&self, action: &Value) {
        tracing::debug!("LangFuse agent action: {}", action);
    }

    fn on_agent_finish(&self, finish: &Value) {
        tracing::debug!("LangFuse agent finish: {}", finish);
    }
}
