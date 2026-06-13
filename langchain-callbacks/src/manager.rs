//! Callback manager — dispatches events to handlers.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use serde_json::Value;
use tracing::warn;

use crate::traits::CallbackHandler;

#[derive(Clone)]
pub struct CallbackManager {
    handlers: Vec<Arc<dyn CallbackHandler>>,
}

impl std::fmt::Debug for CallbackManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackManager")
            .field("handler_count", &self.handlers.len())
            .finish()
    }
}

impl Default for CallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackManager {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(&mut self, handler: Arc<dyn CallbackHandler>) {
        self.handlers.push(handler);
    }

    pub fn set_handlers(&mut self, handlers: Vec<Arc<dyn CallbackHandler>>) {
        self.handlers = handlers;
    }

    pub fn with_handler(mut self, handler: Arc<dyn CallbackHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    pub fn remove_handler(&mut self, idx: usize) -> Option<Arc<dyn CallbackHandler>> {
        if idx < self.handlers.len() {
            Some(self.handlers.remove(idx))
        } else {
            None
        }
    }

    pub fn handlers(&self) -> &[Arc<dyn CallbackHandler>] {
        &self.handlers
    }

    fn run_on_handlers<F>(&self, f: F)
    where
        F: Fn(&Arc<dyn CallbackHandler>) + Send + Sync,
    {
        for handler in &self.handlers {
            let result = catch_unwind(AssertUnwindSafe(|| f(handler)));
            if let Err(err) = result {
                warn!("Callback handler panicked: {:?}", err);
            }
        }
    }

    pub fn on_chain_start(&self, name: &str, inputs: &Value) {
        self.run_on_handlers(|h| h.on_chain_start(name, inputs));
    }

    pub fn on_chain_end(&self, name: &str, outputs: &Value) {
        self.run_on_handlers(|h| h.on_chain_end(name, outputs));
    }

    pub fn on_chain_error(&self, name: &str, error: &Value) {
        self.run_on_handlers(|h| h.on_chain_error(name, error));
    }

    pub fn on_llm_start(&self, name: &str, prompts: &[String]) {
        self.run_on_handlers(|h| h.on_llm_start(name, prompts));
    }

    pub fn on_llm_end(&self, name: &str, output: &Value) {
        self.run_on_handlers(|h| h.on_llm_end(name, output));
    }

    pub fn on_llm_error(&self, name: &str, error: &Value) {
        self.run_on_handlers(|h| h.on_llm_error(name, error));
    }

    pub fn on_llm_new_token(&self, token: &str) {
        self.run_on_handlers(|h| h.on_llm_new_token(token));
    }

    pub fn on_chat_model_start(&self, name: &str, messages: &Value) {
        self.run_on_handlers(|h| h.on_chat_model_start(name, messages));
    }

    pub fn on_tool_start(&self, name: &str, input: &Value) {
        self.run_on_handlers(|h| h.on_tool_start(name, input));
    }

    pub fn on_tool_end(&self, name: &str, output: &Value) {
        self.run_on_handlers(|h| h.on_tool_end(name, output));
    }

    pub fn on_tool_error(&self, name: &str, error: &Value) {
        self.run_on_handlers(|h| h.on_tool_error(name, error));
    }

    pub fn on_retriever_start(&self, query: &str) {
        self.run_on_handlers(|h| h.on_retriever_start(query));
    }

    pub fn on_retriever_end(&self, documents: &Value) {
        self.run_on_handlers(|h| h.on_retriever_end(documents));
    }

    pub fn on_retriever_error(&self, error: &Value) {
        self.run_on_handlers(|h| h.on_retriever_error(error));
    }

    pub fn on_agent_action(&self, action: &Value) {
        self.run_on_handlers(|h| h.on_agent_action(action));
    }

    pub fn on_agent_finish(&self, finish: &Value) {
        self.run_on_handlers(|h| h.on_agent_finish(finish));
    }

    pub fn on_text(&self, text: &str) {
        self.run_on_handlers(|h| h.on_text(text));
    }

    pub fn on_stream(&self, chunk: &Value) {
        self.run_on_handlers(|h| h.on_stream(chunk));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::CallbackHandler;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct CountHandler {
        count: AtomicUsize,
    }

    impl CountHandler {
        fn new() -> Self { Self { count: AtomicUsize::new(0) } }
        fn count(&self) -> usize { self.count.load(Ordering::SeqCst) }
    }

    impl CallbackHandler for CountHandler {
        fn on_chain_start(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_chain_end(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_chain_error(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_llm_start(&self, _: &str, _: &[String]) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_llm_end(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_llm_error(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_llm_new_token(&self, _: &str) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_chat_model_start(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_tool_start(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_tool_end(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_tool_error(&self, _: &str, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_retriever_start(&self, _: &str) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_retriever_end(&self, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_retriever_error(&self, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_agent_action(&self, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_agent_finish(&self, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_text(&self, _: &str) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_stream(&self, _: &Value) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_manager_new_empty() {
        let mgr = CallbackManager::new();
        assert!(mgr.handlers().is_empty());
    }

    #[test]
    fn test_manager_default() {
        let mgr = CallbackManager::default();
        assert!(mgr.handlers().is_empty());
    }

    #[test]
    fn test_manager_add_handler() {
        let mut mgr = CallbackManager::new();
        mgr.add_handler(Arc::new(CountHandler::new()));
        assert_eq!(mgr.handlers().len(), 1);
    }

    #[test]
    fn test_manager_with_handler() {
        let mgr = CallbackManager::new()
            .with_handler(Arc::new(CountHandler::new()))
            .with_handler(Arc::new(CountHandler::new()));
        assert_eq!(mgr.handlers().len(), 2);
    }

    #[test]
    fn test_manager_remove_handler() {
        let mut mgr = CallbackManager::new();
        let h = Arc::new(CountHandler::new());
        mgr.add_handler(h);
        let removed = mgr.remove_handler(0);
        assert!(removed.is_some());
        assert!(mgr.handlers().is_empty());
    }

    #[test]
    fn test_manager_remove_out_of_bounds() {
        let mut mgr = CallbackManager::new();
        assert!(mgr.remove_handler(99).is_none());
    }

    #[test]
    fn test_manager_set_handlers() {
        let mut mgr = CallbackManager::new();
        mgr.set_handlers(vec![
            Arc::new(CountHandler::new()),
            Arc::new(CountHandler::new()),
        ]);
        assert_eq!(mgr.handlers().len(), 2);
    }

    #[test]
    fn test_manager_dispatch_all_events() {
        let h = Arc::new(CountHandler::new());
        let mgr = CallbackManager::new().with_handler(h.clone());
        let v = Value::Null;

        mgr.on_chain_start("c", &v);
        mgr.on_chain_end("c", &v);
        mgr.on_chain_error("c", &v);
        mgr.on_llm_start("c", &[]);
        mgr.on_llm_end("c", &v);
        mgr.on_llm_error("c", &v);
        mgr.on_llm_new_token("tok");
        mgr.on_chat_model_start("c", &v);
        mgr.on_tool_start("c", &v);
        mgr.on_tool_end("c", &v);
        mgr.on_tool_error("c", &v);
        mgr.on_retriever_start("q");
        mgr.on_retriever_end(&v);
        mgr.on_retriever_error(&v);
        mgr.on_agent_action(&v);
        mgr.on_agent_finish(&v);
        mgr.on_text("text");
        mgr.on_stream(&v);

        assert_eq!(h.count(), 18);
    }

    #[test]
    fn test_manager_empty_no_panic() {
        let mgr = CallbackManager::new();
        mgr.on_chain_start("c", &Value::Null);
        mgr.on_llm_new_token("tok");
        mgr.on_text("hello");
    }

    #[test]
    fn test_manager_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CallbackManager>();
        assert_sync::<CallbackManager>();
    }
}
