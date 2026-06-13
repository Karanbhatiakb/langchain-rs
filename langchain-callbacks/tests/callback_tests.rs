use std::sync::{Arc, Mutex};

use langchain_callbacks::manager::CallbackManager;
use langchain_callbacks::traits::CallbackHandler;
use langchain_callbacks::base::BaseCallbackHandler;
use serde_json::Value;

#[cfg(feature = "langfuse")]
use langchain_callbacks::langfuse::LangFuseHandler;
#[cfg(feature = "portkey")]
use langchain_callbacks::portkey::PortkeyHandler;
#[cfg(feature = "comet")]
use langchain_callbacks::comet::CometHandler;
#[cfg(feature = "promptlayer")]
use langchain_callbacks::promptlayer::PromptlayerHandler;
#[cfg(feature = "helicone")]
use langchain_callbacks::helicone::HeliconeHandler;

struct CountingHandler {
    chain_start_count: Arc<Mutex<usize>>,
    chain_end_count: Arc<Mutex<usize>>,
    llm_start_count: Arc<Mutex<usize>>,
    llm_new_token_count: Arc<Mutex<usize>>,
    tool_start_count: Arc<Mutex<usize>>,
    agent_action_count: Arc<Mutex<usize>>,
}

impl CountingHandler {
    fn new() -> Self {
        Self {
            chain_start_count: Arc::new(Mutex::new(0)),
            chain_end_count: Arc::new(Mutex::new(0)),
            llm_start_count: Arc::new(Mutex::new(0)),
            llm_new_token_count: Arc::new(Mutex::new(0)),
            tool_start_count: Arc::new(Mutex::new(0)),
            agent_action_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl CallbackHandler for CountingHandler {
    fn on_chain_start(&self, _name: &str, _inputs: &Value) {
        *self.chain_start_count.lock().unwrap() += 1;
    }
    fn on_chain_end(&self, _name: &str, _outputs: &Value) {
        *self.chain_end_count.lock().unwrap() += 1;
    }
    fn on_llm_start(&self, _name: &str, _prompts: &[String]) {
        *self.llm_start_count.lock().unwrap() += 1;
    }
    fn on_llm_new_token(&self, _token: &str) {
        *self.llm_new_token_count.lock().unwrap() += 1;
    }
    fn on_tool_start(&self, _name: &str, _input: &Value) {
        *self.tool_start_count.lock().unwrap() += 1;
    }
    fn on_agent_action(&self, _action: &Value) {
        *self.agent_action_count.lock().unwrap() += 1;
    }
}

#[test]
fn test_callback_manager_new() {
    let manager = CallbackManager::new();
    assert!(manager.handlers().is_empty());
}

#[test]
fn test_callback_manager_default() {
    let manager = CallbackManager::default();
    assert!(manager.handlers().is_empty());
}

#[test]
fn test_callback_manager_add_handler() {
    let mut manager = CallbackManager::new();
    let handler = BaseCallbackHandler::new();
    manager.add_handler(Arc::new(handler));
    assert_eq!(manager.handlers().len(), 1);
}

#[test]
fn test_callback_manager_set_handlers() {
    let mut manager = CallbackManager::new();
    let handlers: Vec<Arc<dyn CallbackHandler>> = vec![
        Arc::new(BaseCallbackHandler::new()),
        Arc::new(BaseCallbackHandler::new()),
    ];
    manager.set_handlers(handlers);
    assert_eq!(manager.handlers().len(), 2);
}

#[test]
fn test_callback_manager_with_handler() {
    let manager = CallbackManager::new().with_handler(Arc::new(BaseCallbackHandler::new()));
    assert_eq!(manager.handlers().len(), 1);
}

#[test]
fn test_callback_manager_remove_handler() {
    let mut manager = CallbackManager::new();
    manager.add_handler(Arc::new(BaseCallbackHandler::new()));
    manager.add_handler(Arc::new(BaseCallbackHandler::new()));
    let removed = manager.remove_handler(0);
    assert!(removed.is_some());
    assert_eq!(manager.handlers().len(), 1);
}

#[test]
fn test_callback_manager_remove_handler_out_of_bounds() {
    let mut manager = CallbackManager::new();
    let removed = manager.remove_handler(5);
    assert!(removed.is_none());
}

#[test]
fn test_callback_manager_on_chain_start() {
    let handler = CountingHandler::new();
    let chain_count = handler.chain_start_count.clone();
    let manager = CallbackManager::new().with_handler(Arc::new(handler));
    manager.on_chain_start("test", &serde_json::json!({"input": "val"}));
    assert_eq!(*chain_count.lock().unwrap(), 1);
}

#[test]
fn test_callback_manager_on_chain_end() {
    let handler = CountingHandler::new();
    let count = handler.chain_end_count.clone();
    let manager = CallbackManager::new().with_handler(Arc::new(handler));
    manager.on_chain_end("test", &serde_json::json!({"output": "val"}));
    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn test_callback_manager_on_llm_start() {
    let handler = CountingHandler::new();
    let count = handler.llm_start_count.clone();
    let manager = CallbackManager::new().with_handler(Arc::new(handler));
    manager.on_llm_start("gpt-4", &["prompt".to_string()]);
    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn test_callback_manager_on_llm_new_token() {
    let handler = CountingHandler::new();
    let count = handler.llm_new_token_count.clone();
    let manager = CallbackManager::new().with_handler(Arc::new(handler));
    manager.on_llm_new_token("hello");
    manager.on_llm_new_token("world");
    assert_eq!(*count.lock().unwrap(), 2);
}

#[test]
fn test_callback_manager_on_tool_start() {
    let handler = CountingHandler::new();
    let count = handler.tool_start_count.clone();
    let manager = CallbackManager::new().with_handler(Arc::new(handler));
    manager.on_tool_start("search", &serde_json::json!("query"));
    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn test_callback_manager_on_agent_action() {
    let handler = CountingHandler::new();
    let count = handler.agent_action_count.clone();
    let manager = CallbackManager::new().with_handler(Arc::new(handler));
    manager.on_agent_action(&serde_json::json!({"tool": "search"}));
    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn test_callback_manager_multiple_handlers() {
    let h1 = CountingHandler::new();
    let h2 = CountingHandler::new();
    let c1 = h1.chain_start_count.clone();
    let c2 = h2.chain_start_count.clone();
    let manager = CallbackManager::new()
        .with_handler(Arc::new(h1))
        .with_handler(Arc::new(h2));
    manager.on_chain_start("test", &serde_json::json!({}));
    assert_eq!(*c1.lock().unwrap(), 1);
    assert_eq!(*c2.lock().unwrap(), 1);
}

#[test]
fn test_callback_manager_clone() {
    let manager = CallbackManager::new().with_handler(Arc::new(BaseCallbackHandler::new()));
    let cloned = manager.clone();
    assert_eq!(cloned.handlers().len(), 1);
}

#[test]
fn test_base_callback_handler_default() {
    let handler = BaseCallbackHandler::default();
    assert!(handler.on_chain_start.is_none());
    assert!(handler.on_chain_end.is_none());
    assert!(handler.on_llm_start.is_none());
}

#[test]
fn test_base_callback_handler_custom_fn() {
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();
    let handler = BaseCallbackHandler {
        on_chain_start: Some(Arc::new(move |_name, _inputs| {
            *called_clone.lock().unwrap() = true;
        })),
        ..BaseCallbackHandler::new()
    };
    handler.on_chain_start("test", &serde_json::json!({}));
    assert!(*called.lock().unwrap());
}

#[cfg(feature = "langfuse")]
mod langfuse_tests {
    use super::*;
    #[test]
    fn test_langfuse_handler_creation() {
        let handler = LangFuseHandler::new();
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_langfuse_handler_with_api_key() {
        let handler = LangFuseHandler::new().with_api_key("test-key");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_langfuse_handler_with_api_url() {
        let handler = LangFuseHandler::new().with_api_url("https://custom.langfuse.com");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_langfuse_handler_with_project() {
        let handler = LangFuseHandler::new().with_project("my-project");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_langfuse_handler_with_batch_size() {
        let handler = LangFuseHandler::new().with_batch_size(50);
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_langfuse_handler_default() {
        let handler = LangFuseHandler::default();
        handler.on_chain_end("test", &serde_json::json!({}));
    }
    #[test]
    fn test_langfuse_handler_on_llm_start() {
        let handler = LangFuseHandler::new();
        handler.on_llm_start("gpt-4", &["prompt".to_string()]);
    }
    #[test]
    fn test_langfuse_handler_on_tool_start() {
        let handler = LangFuseHandler::new();
        handler.on_tool_start("search", &serde_json::json!("query"));
    }
    #[test]
    fn test_langfuse_handler_on_agent_action() {
        let handler = LangFuseHandler::new();
        handler.on_agent_action(&serde_json::json!({"tool": "search"}));
    }
    #[test]
    fn test_langfuse_handler_on_agent_finish() {
        let handler = LangFuseHandler::new();
        handler.on_agent_finish(&serde_json::json!({"result": "done"}));
    }
}

#[cfg(feature = "portkey")]
mod portkey_tests {
    use super::*;
    #[test]
    fn test_portkey_handler_creation() {
        let handler = PortkeyHandler::new();
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_portkey_handler_with_api_key() {
        let handler = PortkeyHandler::new().with_api_key("test-key");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_portkey_handler_with_base_url() {
        let handler = PortkeyHandler::new().with_base_url("https://custom.portkey.ai");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_portkey_handler_with_config_id() {
        let handler = PortkeyHandler::new().with_config_id("config-1");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_portkey_handler_default() {
        let handler = PortkeyHandler::default();
        handler.on_chain_end("test", &serde_json::json!({}));
    }
    #[test]
    fn test_portkey_handler_on_llm_start() {
        let handler = PortkeyHandler::new();
        handler.on_llm_start("gpt-4", &["prompt".to_string()]);
    }
    #[test]
    fn test_portkey_handler_on_tool_start() {
        let handler = PortkeyHandler::new();
        handler.on_tool_start("search", &serde_json::json!("query"));
    }
    #[test]
    fn test_portkey_handler_on_agent_action() {
        let handler = PortkeyHandler::new();
        handler.on_agent_action(&serde_json::json!({"tool": "search"}));
    }
}

#[cfg(feature = "comet")]
mod comet_tests {
    use super::*;
    #[test]
    fn test_comet_handler_creation() {
        let handler = CometHandler::new();
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_comet_handler_with_api_key() {
        let handler = CometHandler::new().with_api_key("test-key");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_comet_handler_with_project() {
        let handler = CometHandler::new().with_project("my-project");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_comet_handler_with_workspace() {
        let handler = CometHandler::new().with_workspace("my-workspace");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_comet_handler_with_experiment_key() {
        let handler = CometHandler::new().with_experiment_key("exp-123");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_comet_handler_default() {
        let handler = CometHandler::default();
        handler.on_chain_end("test", &serde_json::json!({}));
    }
    #[test]
    fn test_comet_handler_on_llm_end() {
        let handler = CometHandler::new();
        handler.on_llm_end("gpt-4", &serde_json::json!({"tokens": 100}));
    }
    #[test]
    fn test_comet_handler_on_tool_end() {
        let handler = CometHandler::new();
        handler.on_tool_end("search", &serde_json::json!({"result": "found"}));
    }
    #[test]
    fn test_comet_handler_on_agent_action() {
        let handler = CometHandler::new();
        handler.on_agent_action(&serde_json::json!({"tool": "search"}));
    }
    #[test]
    fn test_comet_handler_on_agent_finish() {
        let handler = CometHandler::new();
        handler.on_agent_finish(&serde_json::json!({"result": "done"}));
    }
}

#[cfg(feature = "promptlayer")]
mod promptlayer_tests {
    use super::*;
    #[test]
    fn test_promptlayer_handler_creation() {
        let handler = PromptlayerHandler::new();
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_promptlayer_handler_with_api_key() {
        let handler = PromptlayerHandler::new().with_api_key("test-key");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_promptlayer_handler_with_base_url() {
        let handler = PromptlayerHandler::new().with_base_url("https://custom.promptlayer.com");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_promptlayer_handler_default() {
        let handler = PromptlayerHandler::default();
        handler.on_chain_end("test", &serde_json::json!({}));
    }
    #[test]
    fn test_promptlayer_handler_on_llm_start() {
        let handler = PromptlayerHandler::new();
        handler.on_llm_start("gpt-4", &["prompt".to_string()]);
    }
    #[test]
    fn test_promptlayer_handler_on_tool_start() {
        let handler = PromptlayerHandler::new();
        handler.on_tool_start("search", &serde_json::json!("query"));
    }
    #[test]
    fn test_promptlayer_handler_on_agent_action() {
        let handler = PromptlayerHandler::new();
        handler.on_agent_action(&serde_json::json!({"tool": "search"}));
    }
}

#[cfg(feature = "helicone")]
mod helicone_tests {
    use super::*;
    #[test]
    fn test_helicone_handler_creation() {
        let handler = HeliconeHandler::new();
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_helicone_handler_with_api_key() {
        let handler = HeliconeHandler::new().with_api_key("test-key");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_helicone_handler_with_base_url() {
        let handler = HeliconeHandler::new().with_base_url("https://custom.helicone.ai");
        handler.on_chain_start("test", &serde_json::json!({}));
    }
    #[test]
    fn test_helicone_handler_default() {
        let handler = HeliconeHandler::default();
        handler.on_chain_end("test", &serde_json::json!({}));
    }
    #[test]
    fn test_helicone_handler_on_chain_error() {
        let handler = HeliconeHandler::new();
        handler.on_chain_error("test", &serde_json::json!({"error": "failed"}));
    }
    #[test]
    fn test_helicone_handler_on_llm_start() {
        let handler = HeliconeHandler::new();
        handler.on_llm_start("gpt-4", &["prompt".to_string()]);
    }
    #[test]
    fn test_helicone_handler_on_llm_error() {
        let handler = HeliconeHandler::new();
        handler.on_llm_error("gpt-4", &serde_json::json!({"error": "timeout"}));
    }
    #[test]
    fn test_helicone_handler_on_tool_start() {
        let handler = HeliconeHandler::new();
        handler.on_tool_start("search", &serde_json::json!("query"));
    }
    #[test]
    fn test_helicone_handler_on_tool_error() {
        let handler = HeliconeHandler::new();
        handler.on_tool_error("search", &serde_json::json!({"error": "failed"}));
    }
    #[test]
    fn test_helicone_handler_on_agent_action() {
        let handler = HeliconeHandler::new();
        handler.on_agent_action(&serde_json::json!({"tool": "search"}));
    }
    #[test]
    fn test_helicone_handler_on_agent_finish() {
        let handler = HeliconeHandler::new();
        handler.on_agent_finish(&serde_json::json!({"result": "done"}));
    }
}

#[cfg(all(feature = "langfuse", feature = "portkey", feature = "comet", feature = "promptlayer", feature = "helicone"))]
mod callback_handler_trait_tests {
    use super::*;
    #[test]
    fn test_all_callback_handlers_implement_trait() {
        let manager = CallbackManager::new()
            .with_handler(Arc::new(LangFuseHandler::new()))
            .with_handler(Arc::new(PortkeyHandler::new()))
            .with_handler(Arc::new(CometHandler::new()))
            .with_handler(Arc::new(PromptlayerHandler::new()))
            .with_handler(Arc::new(HeliconeHandler::new()));
        assert_eq!(manager.handlers().len(), 5);
        manager.on_chain_start("test", &serde_json::json!({}));
    }
}
