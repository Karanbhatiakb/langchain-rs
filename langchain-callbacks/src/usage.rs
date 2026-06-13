use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde_json::Value;

use crate::traits::CallbackHandler;

#[derive(Debug, Clone)]
pub struct UsageCallbackHandler {
    total_tokens: Arc<AtomicU64>,
    prompt_tokens: Arc<AtomicU64>,
    completion_tokens: Arc<AtomicU64>,
}

impl UsageCallbackHandler {
    pub fn new() -> Self {
        Self {
            total_tokens: Arc::new(AtomicU64::new(0)),
            prompt_tokens: Arc::new(AtomicU64::new(0)),
            completion_tokens: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_total_tokens(&self) -> u64 {
        self.total_tokens.load(Ordering::SeqCst)
    }

    pub fn get_prompt_tokens(&self) -> u64 {
        self.prompt_tokens.load(Ordering::SeqCst)
    }

    pub fn get_completion_tokens(&self) -> u64 {
        self.completion_tokens.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.total_tokens.store(0, Ordering::SeqCst);
        self.prompt_tokens.store(0, Ordering::SeqCst);
        self.completion_tokens.store(0, Ordering::SeqCst);
    }

    fn extract_u64(value: &Value, key: &str) -> Option<u64> {
        value.get(key).and_then(|v| v.as_u64())
    }
}

impl Default for UsageCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackHandler for UsageCallbackHandler {
    fn on_llm_end(&self, _name: &str, output: &Value) {
        let usage = output
            .get("token_usage")
            .or_else(|| output.get("usage"))
            .or_else(|| output.get("llm_output").and_then(|o| o.get("token_usage")))
            .or_else(|| output.get("llm_output").and_then(|o| o.get("usage")));

        if let Some(usage) = usage {
            if let Some(v) = Self::extract_u64(usage, "total_tokens") {
                self.total_tokens.fetch_add(v, Ordering::SeqCst);
            }
            if let Some(v) = Self::extract_u64(usage, "prompt_tokens") {
                self.prompt_tokens.fetch_add(v, Ordering::SeqCst);
            }
            if let Some(v) = Self::extract_u64(usage, "completion_tokens") {
                self.completion_tokens.fetch_add(v, Ordering::SeqCst);
            }
        }
    }
}
