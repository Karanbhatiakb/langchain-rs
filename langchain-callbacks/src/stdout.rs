//! Stdout callback handler — logs events to console.

use chrono::Local;
use serde_json::Value;

use crate::traits::CallbackHandler;

pub struct StdOutCallbackHandler {
    pub show_time: bool,
    pub color: bool,
}

impl Default for StdOutCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl StdOutCallbackHandler {
    pub fn new() -> Self {
        Self {
            show_time: true,
            color: true,
        }
    }

    pub fn without_time(mut self) -> Self {
        self.show_time = false;
        self
    }

    pub fn without_color(mut self) -> Self {
        self.color = false;
        self
    }

    fn timestamp(&self) -> String {
        if self.show_time {
            format!("[{}] ", Local::now().format("%H:%M:%S%.3f"))
        } else {
            String::new()
        }
    }

    fn colored(&self, text: &str, color_code: &str) -> String {
        if self.color {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    }
}

impl CallbackHandler for StdOutCallbackHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("▶ CHAIN", "36"),
            self.colored(&format!("[{}] inputs={}", name, inputs), "37")
        );
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✓ CHAIN", "32"),
            self.colored(&format!("[{}] outputs={}", name, outputs), "37")
        );
    }

    fn on_chain_error(&self, name: &str, error: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✗ CHAIN", "31"),
            self.colored(&format!("[{}] error={}", name, error), "91")
        );
    }

    fn on_llm_start(&self, name: &str, prompts: &[String]) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("▶ LLM", "36"),
            self.colored(&format!("[{}] {} prompt(s)", name, prompts.len()), "37")
        );
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✓ LLM", "32"),
            self.colored(&format!("[{}] {}", name, output), "37")
        );
    }

    fn on_llm_error(&self, name: &str, error: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✗ LLM", "31"),
            self.colored(&format!("[{}] {}", name, error), "91")
        );
    }

    fn on_llm_new_token(&self, token: &str) {
        print!("{}", token);
    }

    fn on_chat_model_start(&self, name: &str, messages: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("▶ CHAT", "36"),
            self.colored(&format!("[{}] {}", name, messages), "37")
        );
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("▶ TOOL", "36"),
            self.colored(&format!("[{}] input={}", name, input), "37")
        );
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✓ TOOL", "32"),
            self.colored(&format!("[{}] output={}", name, output), "37")
        );
    }

    fn on_tool_error(&self, name: &str, error: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✗ TOOL", "31"),
            self.colored(&format!("[{}] {}", name, error), "91")
        );
    }

    fn on_retriever_start(&self, query: &str) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("▶ RETRIEVER", "36"),
            self.colored(&format!("query=\"{}\"", query), "37")
        );
    }

    fn on_retriever_end(&self, documents: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✓ RETRIEVER", "32"),
            self.colored(&format!("docs={}", documents), "37")
        );
    }

    fn on_retriever_error(&self, error: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✗ RETRIEVER", "31"),
            self.colored(&format!("error={}", error), "91")
        );
    }

    fn on_agent_action(&self, action: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("→ AGENT ACTION", "33"),
            self.colored(&format!("{}", action), "93")
        );
    }

    fn on_agent_finish(&self, finish: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("✓ AGENT FINISH", "32"),
            self.colored(&format!("{}", finish), "37")
        );
    }

    fn on_text(&self, text: &str) {
        println!("{}{}", self.timestamp(), text);
    }

    fn on_stream(&self, chunk: &Value) {
        println!(
            "{} {} {}",
            self.timestamp(),
            self.colored("STREAM", "35"),
            chunk
        );
    }
}
