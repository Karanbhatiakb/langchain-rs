//! Core [`Chain`] trait for composing LLM calls.

use async_trait::async_trait;
use langchain_core::errors::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Trait for callable chain components that accept named inputs and produce
/// named outputs.
#[async_trait]
pub trait Chain: Send + Sync {
    /// Returns the list of input key names expected by this chain.
    fn input_keys(&self) -> Vec<String>;
    /// Returns the list of output key names produced by this chain.
    fn output_keys(&self) -> Vec<String>;
    /// Executes the chain with the given inputs and returns the outputs.
    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct TestChain;

    #[async_trait]
    impl Chain for TestChain {
        fn input_keys(&self) -> Vec<String> {
            vec!["input".into()]
        }
        fn output_keys(&self) -> Vec<String> {
            vec!["output".into()]
        }
        async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
            let input = inputs.get("input").cloned().unwrap_or(Value::Null);
            let mut result = HashMap::new();
            result.insert("output".into(), input);
            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_chain_input_keys() {
        let chain = TestChain;
        let keys = chain.input_keys();
        assert_eq!(keys, vec!["input"]);
    }

    #[tokio::test]
    async fn test_chain_output_keys() {
        let chain = TestChain;
        let keys = chain.output_keys();
        assert_eq!(keys, vec!["output"]);
    }

    #[tokio::test]
    async fn test_chain_call() {
        let chain = TestChain;
        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("hello".into()));
        let result = chain.call(inputs).await.unwrap();
        assert_eq!(
            result.get("output").unwrap().as_str().unwrap(),
            "hello"
        );
    }

    #[tokio::test]
    async fn test_chain_call_missing_input() {
        let chain = TestChain;
        let inputs = HashMap::new();
        let result = chain.call(inputs).await.unwrap();
        assert_eq!(result.get("output").unwrap(), &Value::Null);
    }

    #[test]
    fn test_chain_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<TestChain>();
        assert_sync::<TestChain>();
    }
}
