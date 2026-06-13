//! Base callback handler and async callback handler traits.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::traits::CallbackHandler;

pub struct BaseCallbackHandler {
    pub on_chain_start: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_chain_end: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_chain_error: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_llm_start: Option<Arc<dyn Fn(&str, &[String]) + Send + Sync>>,
    pub on_llm_end: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_llm_error: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_llm_new_token: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    pub on_chat_model_start: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_tool_start: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_tool_end: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_tool_error: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    pub on_retriever_start: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    pub on_retriever_end: Option<Arc<dyn Fn(&Value) + Send + Sync>>,
    pub on_retriever_error: Option<Arc<dyn Fn(&Value) + Send + Sync>>,
    pub on_agent_action: Option<Arc<dyn Fn(&Value) + Send + Sync>>,
    pub on_agent_finish: Option<Arc<dyn Fn(&Value) + Send + Sync>>,
    pub on_text: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    pub on_stream: Option<Arc<dyn Fn(&Value) + Send + Sync>>,
}

impl Default for BaseCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseCallbackHandler {
    pub fn new() -> Self {
        Self {
            on_chain_start: None,
            on_chain_end: None,
            on_chain_error: None,
            on_llm_start: None,
            on_llm_end: None,
            on_llm_error: None,
            on_llm_new_token: None,
            on_chat_model_start: None,
            on_tool_start: None,
            on_tool_end: None,
            on_tool_error: None,
            on_retriever_start: None,
            on_retriever_end: None,
            on_retriever_error: None,
            on_agent_action: None,
            on_agent_finish: None,
            on_text: None,
            on_stream: None,
        }
    }
}

impl CallbackHandler for BaseCallbackHandler {
    fn on_chain_start(&self, name: &str, inputs: &Value) {
        if let Some(ref f) = self.on_chain_start {
            f(name, inputs);
        }
    }

    fn on_chain_end(&self, name: &str, outputs: &Value) {
        if let Some(ref f) = self.on_chain_end {
            f(name, outputs);
        }
    }

    fn on_chain_error(&self, name: &str, error: &Value) {
        if let Some(ref f) = self.on_chain_error {
            f(name, error);
        }
    }

    fn on_llm_start(&self, name: &str, prompts: &[String]) {
        if let Some(ref f) = self.on_llm_start {
            f(name, prompts);
        }
    }

    fn on_llm_end(&self, name: &str, output: &Value) {
        if let Some(ref f) = self.on_llm_end {
            f(name, output);
        }
    }

    fn on_llm_error(&self, name: &str, error: &Value) {
        if let Some(ref f) = self.on_llm_error {
            f(name, error);
        }
    }

    fn on_llm_new_token(&self, token: &str) {
        if let Some(ref f) = self.on_llm_new_token {
            f(token);
        }
    }

    fn on_chat_model_start(&self, name: &str, messages: &Value) {
        if let Some(ref f) = self.on_chat_model_start {
            f(name, messages);
        }
    }

    fn on_tool_start(&self, name: &str, input: &Value) {
        if let Some(ref f) = self.on_tool_start {
            f(name, input);
        }
    }

    fn on_tool_end(&self, name: &str, output: &Value) {
        if let Some(ref f) = self.on_tool_end {
            f(name, output);
        }
    }

    fn on_tool_error(&self, name: &str, error: &Value) {
        if let Some(ref f) = self.on_tool_error {
            f(name, error);
        }
    }

    fn on_retriever_start(&self, query: &str) {
        if let Some(ref f) = self.on_retriever_start {
            f(query);
        }
    }

    fn on_retriever_end(&self, documents: &Value) {
        if let Some(ref f) = self.on_retriever_end {
            f(documents);
        }
    }

    fn on_retriever_error(&self, error: &Value) {
        if let Some(ref f) = self.on_retriever_error {
            f(error);
        }
    }

    fn on_agent_action(&self, action: &Value) {
        if let Some(ref f) = self.on_agent_action {
            f(action);
        }
    }

    fn on_agent_finish(&self, finish: &Value) {
        if let Some(ref f) = self.on_agent_finish {
            f(finish);
        }
    }

    fn on_text(&self, text: &str) {
        if let Some(ref f) = self.on_text {
            f(text);
        }
    }

    fn on_stream(&self, chunk: &Value) {
        if let Some(ref f) = self.on_stream {
            f(chunk);
        }
    }
}

#[async_trait]
pub trait AsyncCallbackHandler: Send + Sync {
    async fn on_chain_start(&self, _name: &str, _inputs: &Value) {}
    async fn on_chain_end(&self, _name: &str, _outputs: &Value) {}
    async fn on_chain_error(&self, _name: &str, _error: &Value) {}
    async fn on_llm_start(&self, _name: &str, _prompts: &[String]) {}
    async fn on_llm_end(&self, _name: &str, _output: &Value) {}
    async fn on_llm_error(&self, _name: &str, _error: &Value) {}
    async fn on_llm_new_token(&self, _token: &str) {}
    async fn on_chat_model_start(&self, _name: &str, _messages: &Value) {}
    async fn on_tool_start(&self, _name: &str, _input: &Value) {}
    async fn on_tool_end(&self, _name: &str, _output: &Value) {}
    async fn on_tool_error(&self, _name: &str, _error: &Value) {}
    async fn on_retriever_start(&self, _query: &str) {}
    async fn on_retriever_end(&self, _documents: &Value) {}
    async fn on_retriever_error(&self, _error: &Value) {}
    async fn on_agent_action(&self, _action: &Value) {}
    async fn on_agent_finish(&self, _finish: &Value) {}
    async fn on_text(&self, _text: &str) {}
    async fn on_stream(&self, _chunk: &Value) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_base_callback_handler_default() {
        let h = BaseCallbackHandler::new();
        assert!(h.on_chain_start.is_none());
        assert!(h.on_chain_end.is_none());
        assert!(h.on_chain_error.is_none());
        assert!(h.on_llm_start.is_none());
        assert!(h.on_llm_end.is_none());
        assert!(h.on_llm_error.is_none());
        assert!(h.on_llm_new_token.is_none());
        assert!(h.on_chat_model_start.is_none());
        assert!(h.on_tool_start.is_none());
        assert!(h.on_tool_end.is_none());
        assert!(h.on_tool_error.is_none());
        assert!(h.on_retriever_start.is_none());
        assert!(h.on_retriever_end.is_none());
        assert!(h.on_retriever_error.is_none());
        assert!(h.on_agent_action.is_none());
        assert!(h.on_agent_finish.is_none());
        assert!(h.on_text.is_none());
        assert!(h.on_stream.is_none());
    }

    #[test]
    fn test_base_callback_handler_with_closures() {
        let chain_was_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let chain_was_called_clone = chain_was_called.clone();
        let on_chain = Arc::new(move |_: &str, _: &Value| {
            chain_was_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        let mut h = BaseCallbackHandler::new();
        h.on_chain_start = Some(on_chain);

        h.on_chain_start("test", &Value::Null);
        assert!(chain_was_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_base_handler_implements_callback_handler() {
        fn takes_handler(_: &dyn CallbackHandler) {}
        let h = BaseCallbackHandler::new();
        takes_handler(&h);
    }

    #[test]
    fn test_base_handler_default_no_panic() {
        let h = BaseCallbackHandler::new();
        h.on_chain_start("c", &Value::Null);
        h.on_chain_end("c", &Value::Null);
        h.on_chain_error("c", &Value::Null);
        h.on_llm_start("c", &[]);
        h.on_llm_end("c", &Value::Null);
        h.on_llm_error("c", &Value::Null);
        h.on_llm_new_token("tok");
        h.on_chat_model_start("c", &Value::Null);
        h.on_tool_start("c", &Value::Null);
        h.on_tool_end("c", &Value::Null);
        h.on_tool_error("c", &Value::Null);
        h.on_retriever_start("q");
        h.on_retriever_end(&Value::Null);
        h.on_retriever_error(&Value::Null);
        h.on_agent_action(&Value::Null);
        h.on_agent_finish(&Value::Null);
        h.on_text("text");
        h.on_stream(&Value::Null);
    }

    #[test]
    fn test_async_callback_handler_defaults() {
        struct EmptyAsync;
        #[async_trait]
        impl AsyncCallbackHandler for EmptyAsync {}
        let h = EmptyAsync;
        let v = Value::Null;
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            h.on_chain_start("c", &v).await;
            h.on_chain_end("c", &v).await;
            h.on_chain_error("c", &v).await;
            h.on_llm_start("c", &[]).await;
            h.on_llm_end("c", &v).await;
            h.on_llm_error("c", &v).await;
            h.on_llm_new_token("tok").await;
            h.on_chat_model_start("c", &v).await;
            h.on_tool_start("c", &v).await;
            h.on_tool_end("c", &v).await;
            h.on_tool_error("c", &v).await;
            h.on_retriever_start("q").await;
            h.on_retriever_end(&v).await;
            h.on_retriever_error(&v).await;
            h.on_agent_action(&v).await;
            h.on_agent_finish(&v).await;
            h.on_text("text").await;
            h.on_stream(&v).await;
        });
    }

    #[test]
    fn test_base_handler_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<BaseCallbackHandler>();
        assert_sync::<BaseCallbackHandler>();
        assert_send::<Box<dyn AsyncCallbackHandler>>();
        assert_sync::<Box<dyn AsyncCallbackHandler>>();
    }
}
