//! streaming_demo module.

pub async fn run() -> anyhow::Result<()> {
    println!("Streaming LLM responses (simulated):");

    let tokens = vec![
        "Hello", " ", "from", " ", "the", " ", "streaming", " ", "LLM", "!",
    ];

    for token in &tokens {
        print!("{}", token);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    println!();

    println!("\nStreaming via CallbackHandler:");
    let handler = Arc::new(StreamCallbackHandler);
    let cb_manager = langchain_callbacks::CallbackManager::new()
        .with_handler(handler);

    let prompts = vec!["Tell me a story".to_string()];
    cb_manager.on_llm_start("StreamingLLM", &prompts);

    for token in &["Once", " ", "upon", " ", "a", " ", "time", "..."] {
        cb_manager.on_llm_new_token(token);
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
    }

    let output = serde_json::json!({"text": "Once upon a time..."});
    cb_manager.on_llm_end("StreamingLLM", &output);

    Ok(())
}

use std::sync::Arc;
use langchain_callbacks::CallbackHandler;

struct StreamCallbackHandler;

impl CallbackHandler for StreamCallbackHandler {
    fn on_llm_new_token(&self, token: &str) {
        print!("{}", token);
    }

    fn on_llm_start(&self, name: &str, _prompts: &[String]) {
        println!("  [Stream] LLM '{}' started", name);
    }

    fn on_llm_end(&self, name: &str, _output: &serde_json::Value) {
        println!("\n  [Stream] LLM '{}' ended", name);
    }
}
