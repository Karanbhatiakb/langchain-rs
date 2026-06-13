//! callbacks_demo module.

use std::sync::Arc;

use langchain_callbacks::traits::CallbackHandler;
use langchain_callbacks::CallbackManager;

pub async fn run() -> anyhow::Result<()> {
    let mut manager = CallbackManager::new();

    let stdout_handler = Arc::new(langchain_callbacks::stdout::StdOutCallbackHandler::new());
    manager.add_handler(stdout_handler);

    let custom = Arc::new(CustomHandler::new("Handler1"));
    manager.add_handler(custom);

    let input = serde_json::json!({"question": "What is Rust?"});
    manager.on_chain_start("DemoChain", &input);

    let prompts = vec!["Tell me about Rust".to_string()];
    manager.on_llm_start("GPT-4", &prompts);

    manager.on_llm_new_token("Rust");
    manager.on_llm_new_token(" is ");
    manager.on_llm_new_token("awesome!");

    let output = serde_json::json!({"text": "Rust is a systems language."});
    manager.on_llm_end("GPT-4", &output);

    let tool_input = serde_json::json!({"arg": "2+2"});
    manager.on_tool_start("calculator", &tool_input);
    let tool_output = serde_json::json!({"result": "4"});
    manager.on_tool_end("calculator", &tool_output);

    let action = serde_json::json!({"tool": "calculator", "input": "2+2"});
    manager.on_agent_action(&action);

    let finish = serde_json::json!({"output": "The answer is 4"});
    manager.on_agent_finish(&finish);

    let docs = serde_json::json!([{"page_content": "Rust doc"}]);
    manager.on_retriever_start("query_rust");
    manager.on_retriever_end(&docs);

    manager.on_chain_end("DemoChain", &output);

    let error = serde_json::json!({"message": "Something went wrong"});
    manager.on_chain_error("FailingChain", &error);

    Ok(())
}

struct CustomHandler {
    name: String,
}

impl CustomHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl CallbackHandler for CustomHandler {
    fn on_chain_start(&self, name: &str, _inputs: &serde_json::Value) {
        println!("  [{}] Chain '{}' started", self.name, name);
    }

    fn on_chain_end(&self, name: &str, _outputs: &serde_json::Value) {
        println!("  [{}] Chain '{}' ended", self.name, name);
    }

    fn on_llm_new_token(&self, token: &str) {
        print!("{}", token);
    }
}
