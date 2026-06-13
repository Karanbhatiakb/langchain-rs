use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::traits::CallbackHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub name: String,
    pub run_type: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub inputs: Option<serde_json::Value>,
    pub outputs: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct RunCollectorCallbackHandler {
    runs: Arc<Mutex<Vec<RunRecord>>>,
}

impl RunCollectorCallbackHandler {
    pub fn new() -> Self {
        Self {
            runs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_runs(&self) -> Vec<RunRecord> {
        self.runs.lock().unwrap().clone()
    }

    fn add_start(&self, name: &str, run_type: &str, inputs: Option<Value>) {
        let record = RunRecord {
            name: name.to_string(),
            run_type: run_type.to_string(),
            start_time: Some(chrono::Local::now().to_rfc3339()),
            end_time: None,
            inputs,
            outputs: None,
            error: None,
        };
        self.runs.lock().unwrap().push(record);
    }

    fn update_end(&self, name: &str, run_type: &str, outputs: Option<Value>, error: Option<Value>) {
        let mut runs = self.runs.lock().unwrap();
        if let Some(record) = runs.iter_mut().rev().find(|r| {
            r.name == name && r.run_type == run_type && r.end_time.is_none()
        }) {
            record.end_time = Some(chrono::Local::now().to_rfc3339());
            record.outputs = outputs;
            record.error = error;
        }
    }
}

impl Default for RunCollectorCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackHandler for RunCollectorCallbackHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        self.add_start(name, "chain", Some(inputs.clone()));
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        self.update_end(name, "chain", Some(outputs.clone()), None);
    }

    fn on_chain_error(&self, name: &str, error: &Value) {
        self.update_end(name, "chain", None, Some(error.clone()));
    }

    fn on_llm_start(&self, name: &str, _prompts: &[String]) {
        self.add_start(name, "llm", None);
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        self.update_end(name, "llm", Some(output.clone()), None);
    }

    fn on_llm_error(&self, name: &str, error: &Value) {
        self.update_end(name, "llm", None, Some(error.clone()));
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        self.add_start(name, "tool", Some(input.clone()));
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        self.update_end(name, "tool", Some(output.clone()), None);
    }

    fn on_tool_error(&self, name: &str, error: &Value) {
        self.update_end(name, "tool", None, Some(error.clone()));
    }
}
