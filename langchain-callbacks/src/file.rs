use std::fs::OpenOptions;
use std::io::Write;

use chrono::Local;
use serde_json::Value;

use crate::traits::CallbackHandler;

#[derive(Debug, Clone)]
pub struct FileCallbackHandler {
    path: String,
}

impl FileCallbackHandler {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    fn write_line(&self, event: &str, data: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f");
            let _ = writeln!(file, "[{}] {}: {}", timestamp, event, data);
        }
    }
}

impl CallbackHandler for FileCallbackHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        self.write_line("chain_start", &format!("name={}, inputs={}", name, inputs));
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        self.write_line("chain_end", &format!("name={}, outputs={}", name, outputs));
    }

    fn on_chain_error(&self, name: &str, error: &Value) {
        self.write_line("chain_error", &format!("name={}, error={}", name, error));
    }

    fn on_llm_start(&self, name: &str, prompts: &[String]) {
        self.write_line("llm_start", &format!("name={}, prompts={:?}", name, prompts));
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        self.write_line("llm_end", &format!("name={}, output={}", name, output));
    }

    fn on_llm_error(&self, name: &str, error: &Value) {
        self.write_line("llm_error", &format!("name={}, error={}", name, error));
    }

    fn on_llm_new_token(&self, token: &str) {
        self.write_line("llm_new_token", token);
    }

    fn on_chat_model_start(&self, name: &str, messages: &Value) {
        self.write_line("chat_model_start", &format!("name={}, messages={}", name, messages));
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        self.write_line("tool_start", &format!("name={}, input={}", name, input));
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        self.write_line("tool_end", &format!("name={}, output={}", name, output));
    }

    fn on_tool_error(&self, name: &str, error: &Value) {
        self.write_line("tool_error", &format!("name={}, error={}", name, error));
    }

    fn on_retriever_start(&self, query: &str) {
        self.write_line("retriever_start", query);
    }

    fn on_retriever_end(&self, documents: &Value) {
        self.write_line("retriever_end", &format!("documents={}", documents));
    }

    fn on_retriever_error(&self, error: &Value) {
        self.write_line("retriever_error", &format!("error={}", error));
    }

    fn on_agent_action(&self, action: &Value) {
        self.write_line("agent_action", &format!("action={}", action));
    }

    fn on_agent_finish(&self, finish: &Value) {
        self.write_line("agent_finish", &format!("finish={}", finish));
    }

    fn on_text(&self, text: &str) {
        self.write_line("text", text);
    }

    fn on_stream(&self, chunk: &Value) {
        self.write_line("stream", &format!("chunk={}", chunk));
    }
}
