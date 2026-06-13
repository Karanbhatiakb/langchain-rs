use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use crate::traits::CallbackHandler;

#[derive(Debug, Clone)]
pub struct EventStreamCallbackHandler {
    events: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl EventStreamCallbackHandler {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_events(&self) -> Vec<serde_json::Value> {
        self.events.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }

    fn push_event(&self, event_type: &str, data: Value) {
        let event = json!({
            "event": event_type,
            "data": data,
            "timestamp": chrono::Local::now().to_rfc3339(),
        });
        self.events.lock().unwrap().push(event);
    }
}

impl Default for EventStreamCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackHandler for EventStreamCallbackHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        self.push_event("chain_start", json!({"name": name, "inputs": inputs}));
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        self.push_event("chain_end", json!({"name": name, "outputs": outputs}));
    }

    fn on_chain_error(&self, name: &str, error: &Value) {
        self.push_event("chain_error", json!({"name": name, "error": error}));
    }

    fn on_llm_start(&self, name: &str, prompts: &[String]) {
        self.push_event("llm_start", json!({"name": name, "prompt_count": prompts.len()}));
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        self.push_event("llm_end", json!({"name": name, "output": output}));
    }

    fn on_llm_error(&self, name: &str, error: &Value) {
        self.push_event("llm_error", json!({"name": name, "error": error}));
    }

    fn on_llm_new_token(&self, token: &str) {
        self.push_event("llm_new_token", json!({"token": token}));
    }

    fn on_chat_model_start(&self, name: &str, messages: &Value) {
        self.push_event("chat_model_start", json!({"name": name, "messages": messages}));
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        self.push_event("tool_start", json!({"name": name, "input": input}));
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        self.push_event("tool_end", json!({"name": name, "output": output}));
    }

    fn on_tool_error(&self, name: &str, error: &Value) {
        self.push_event("tool_error", json!({"name": name, "error": error}));
    }

    fn on_retriever_start(&self, query: &str) {
        self.push_event("retriever_start", json!({"query": query}));
    }

    fn on_retriever_end(&self, documents: &Value) {
        self.push_event("retriever_end", json!({"documents": documents}));
    }

    fn on_retriever_error(&self, error: &Value) {
        self.push_event("retriever_error", json!({"error": error}));
    }

    fn on_agent_action(&self, action: &Value) {
        self.push_event("agent_action", json!({"action": action}));
    }

    fn on_agent_finish(&self, finish: &Value) {
        self.push_event("agent_finish", json!({"finish": finish}));
    }

    fn on_text(&self, text: &str) {
        self.push_event("text", json!({"text": text}));
    }

    fn on_stream(&self, chunk: &Value) {
        self.push_event("stream", json!({"chunk": chunk}));
    }
}
