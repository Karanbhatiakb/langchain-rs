use serde_json::Value;

use crate::traits::CallbackHandler;

#[derive(Debug, Clone, Default)]
pub struct StreamingStdOutCallbackHandler;

impl StreamingStdOutCallbackHandler {
    pub fn new() -> Self {
        Self
    }
}

impl CallbackHandler for StreamingStdOutCallbackHandler {
    fn on_llm_new_token(&self, token: &str) {
        print!("{}", token);
    }

    fn on_llm_end(&self, _name: &str, _output: &Value) {
        println!();
    }
}
