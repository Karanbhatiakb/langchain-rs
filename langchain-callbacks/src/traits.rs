//! Core [`CallbackHandler`] trait for observing LangChain lifecycle events.

use serde_json::Value;

/// Trait for handling lifecycle events during LangChain execution.
///
/// All methods have default no-op implementations. Override only the events
/// you care about.
pub trait CallbackHandler: Send + Sync {
    /// Called when a chain run starts.
    fn on_chain_start(&self, _name: &str, _inputs: &Value) {}
    /// Called when a chain run finishes.
    fn on_chain_end(&self, _name: &str, _outputs: &Value) {}
    /// Called when a chain run errors.
    fn on_chain_error(&self, _name: &str, _error: &Value) {}
    /// Called when an LLM starts generating.
    fn on_llm_start(&self, _name: &str, _prompts: &[String]) {}
    /// Called when an LLM finishes generating.
    fn on_llm_end(&self, _name: &str, _output: &Value) {}
    /// Called when an LLM errors.
    fn on_llm_error(&self, _name: &str, _error: &Value) {}
    /// Called for each new token from an LLM stream.
    fn on_llm_new_token(&self, _token: &str) {}
    /// Called when a chat model starts.
    fn on_chat_model_start(&self, _name: &str, _messages: &Value) {}
    /// Called when a tool starts executing.
    fn on_tool_start(&self, _name: &str, _input: &Value) {}
    /// Called when a tool finishes.
    fn on_tool_end(&self, _name: &str, _output: &Value) {}
    /// Called when a tool errors.
    fn on_tool_error(&self, _name: &str, _error: &Value) {}
    /// Called when a retriever starts fetching documents.
    fn on_retriever_start(&self, _query: &str) {}
    /// Called when a retriever finishes.
    fn on_retriever_end(&self, _documents: &Value) {}
    /// Called when a retriever errors.
    fn on_retriever_error(&self, _error: &Value) {}
    /// Called when an agent takes an action.
    fn on_agent_action(&self, _action: &Value) {}
    /// Called when an agent finishes.
    fn on_agent_finish(&self, _finish: &Value) {}
    /// Called for general text logging.
    fn on_text(&self, _text: &str) {}
    /// Called for each streaming chunk.
    fn on_stream(&self, _chunk: &Value) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct TestHandler {
        called: AtomicBool,
    }

    impl CallbackHandler for TestHandler {
        fn on_chain_start(&self, name: &str, _inputs: &Value) {
            assert_eq!(name, "test_chain");
            self.called.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_handler_defaults() {
        struct Empty;
        impl CallbackHandler for Empty {}
        let h = Empty;
        h.on_chain_start("x", &Value::Null);
        h.on_llm_new_token("tok");
    }

    #[test]
    fn test_handler_called() {
        let h = TestHandler { called: AtomicBool::new(false) };
        h.on_chain_start("test_chain", &Value::Null);
        assert!(h.called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_all_methods_no_panic() {
        struct AllMethods;
        impl CallbackHandler for AllMethods {
            fn on_chain_start(&self, _: &str, _: &Value) {}
            fn on_chain_end(&self, _: &str, _: &Value) {}
            fn on_chain_error(&self, _: &str, _: &Value) {}
            fn on_llm_start(&self, _: &str, _: &[String]) {}
            fn on_llm_end(&self, _: &str, _: &Value) {}
            fn on_llm_error(&self, _: &str, _: &Value) {}
            fn on_llm_new_token(&self, _: &str) {}
            fn on_chat_model_start(&self, _: &str, _: &Value) {}
            fn on_tool_start(&self, _: &str, _: &Value) {}
            fn on_tool_end(&self, _: &str, _: &Value) {}
            fn on_tool_error(&self, _: &str, _: &Value) {}
            fn on_retriever_start(&self, _: &str) {}
            fn on_retriever_end(&self, _: &Value) {}
            fn on_retriever_error(&self, _: &Value) {}
            fn on_agent_action(&self, _: &Value) {}
            fn on_agent_finish(&self, _: &Value) {}
            fn on_text(&self, _: &str) {}
            fn on_stream(&self, _: &Value) {}
        }
        let h = AllMethods;
        let v = Value::Null;
        h.on_chain_start("c", &v);
        h.on_chain_end("c", &v);
        h.on_chain_error("c", &v);
        h.on_llm_start("c", &[]);
        h.on_llm_end("c", &v);
        h.on_llm_error("c", &v);
        h.on_llm_new_token("tok");
        h.on_chat_model_start("c", &v);
        h.on_tool_start("c", &v);
        h.on_tool_end("c", &v);
        h.on_tool_error("c", &v);
        h.on_retriever_start("q");
        h.on_retriever_end(&v);
        h.on_retriever_error(&v);
        h.on_agent_action(&v);
        h.on_agent_finish(&v);
        h.on_text("text");
        h.on_stream(&v);
    }

    #[test]
    fn test_handler_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<TestHandler>();
        assert_sync::<TestHandler>();
    }
}
