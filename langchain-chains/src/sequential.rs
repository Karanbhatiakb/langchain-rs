//! Sequential chain implementation.

use async_trait::async_trait;
use langchain_core::errors::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct SimpleSequentialChain {
    chains: Vec<Arc<dyn Chain>>,
    verbose: bool,
}

impl SimpleSequentialChain {
    pub fn new(chains: Vec<Arc<dyn Chain>>) -> Self {
        Self {
            chains,
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn add_chain(mut self, chain: Arc<dyn Chain>) -> Self {
        self.chains.push(chain);
        self
    }
}

#[async_trait]
impl Chain for SimpleSequentialChain {
    fn input_keys(&self) -> Vec<String> {
        self.chains
            .first()
            .map(|c| c.input_keys())
            .unwrap_or_default()
    }

    fn output_keys(&self) -> Vec<String> {
        self.chains
            .last()
            .map(|c| c.output_keys())
            .unwrap_or_default()
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut current_input = inputs;

        for (i, chain) in self.chains.iter().enumerate() {
            if self.verbose {
                info!("SimpleSequentialChain running chain {} with input: {:?}", i, current_input);
            }
            current_input = chain.call(current_input).await?;
        }

        Ok(current_input)
    }
}

pub struct SequentialChain {
    chains: Vec<Arc<dyn Chain>>,
    input_variables: Vec<String>,
    output_variables: Vec<String>,
    verbose: bool,
}

impl SequentialChain {
    pub fn new(
        chains: Vec<Arc<dyn Chain>>,
        input_variables: Vec<String>,
        output_variables: Vec<String>,
    ) -> Self {
        Self {
            chains,
            input_variables,
            output_variables,
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[async_trait]
impl Chain for SequentialChain {
    fn input_keys(&self) -> Vec<String> {
        self.input_variables.clone()
    }

    fn output_keys(&self) -> Vec<String> {
        self.output_variables.clone()
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut all_outputs = inputs;

        for (i, chain) in self.chains.iter().enumerate() {
            if self.verbose {
                info!("SequentialChain running chain {} with accumulated state: {:?}", i, all_outputs);
            }

            let chain_inputs: HashMap<String, Value> = chain
                .input_keys()
                .iter()
                .filter_map(|k| {
                    all_outputs
                        .get(k)
                        .map(|v| (k.clone(), v.clone()))
                })
                .collect();

            let chain_outputs = chain.call(chain_inputs).await?;
            all_outputs.extend(chain_outputs);
        }

        Ok(all_outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;

    struct EchoChain;

    #[async_trait]
    impl Chain for EchoChain {
        fn input_keys(&self) -> Vec<String> { vec!["input".into()] }
        fn output_keys(&self) -> Vec<String> { vec!["output".into()] }
        async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
            let val = inputs.get("input").cloned().unwrap_or(Value::Null);
            Ok(HashMap::from([("output".into(), val)]))
        }
    }

    #[tokio::test]
    async fn test_simple_sequential_chain_single() {
        let chain = SimpleSequentialChain::new(vec![Arc::new(EchoChain)]);
        let mut input = HashMap::new();
        input.insert("input".into(), Value::String("hello".into()));
        let result = chain.call(input).await.unwrap();
        assert_eq!(result.get("output").and_then(|v| v.as_str()), Some("hello"));
    }

    #[tokio::test]
    async fn test_simple_sequential_chain_chained() {
        let chain = SimpleSequentialChain::new(vec![
            Arc::new(EchoChain),
            Arc::new(EchoChain),
        ]);
        let mut input = HashMap::new();
        input.insert("input".into(), Value::String("pass".into()));
        let result = chain.call(input).await.unwrap();
        assert_eq!(result.get("output").and_then(|v| v.as_str()), Some("pass"));
    }

    #[tokio::test]
    async fn test_simple_sequential_chain_input_keys() {
        let chain = SimpleSequentialChain::new(vec![Arc::new(EchoChain)]);
        assert_eq!(chain.input_keys(), vec!["input"]);
    }

    #[tokio::test]
    async fn test_simple_sequential_chain_output_keys() {
        let chain = SimpleSequentialChain::new(vec![Arc::new(EchoChain)]);
        assert_eq!(chain.output_keys(), vec!["output"]);
    }

    #[tokio::test]
    async fn test_simple_sequential_chain_empty() {
        let chain = SimpleSequentialChain::new(vec![]);
        let input = HashMap::new();
        let result = chain.call(input).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_simple_sequential_chain_add_chain() {
        let chain = SimpleSequentialChain::new(vec![]).add_chain(Arc::new(EchoChain));
        assert_eq!(chain.input_keys(), vec!["input"]);
    }

    #[tokio::test]
    async fn test_simple_sequential_with_verbose() {
        let chain = SimpleSequentialChain::new(vec![Arc::new(EchoChain)]).with_verbose(true);
        let mut input = HashMap::new();
        input.insert("input".into(), Value::String("test".into()));
        let result = chain.call(input).await.unwrap();
        assert!(result.contains_key("output"));
    }

    #[tokio::test]
    async fn test_sequential_chain_basic() {
        let chain = SequentialChain::new(
            vec![Arc::new(EchoChain)],
            vec!["input".into()],
            vec!["output".into()],
        );
        assert_eq!(chain.input_keys(), vec!["input"]);
        assert_eq!(chain.output_keys(), vec!["output"]);
    }

    #[tokio::test]
    async fn test_sequential_chain_multiple_io() {
        struct AddKeyChain;
        #[async_trait]
        impl Chain for AddKeyChain {
            fn input_keys(&self) -> Vec<String> { vec!["a".into()] }
            fn output_keys(&self) -> Vec<String> { vec!["b".into()] }
            async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
                let a = inputs.get("a").cloned().unwrap_or(Value::Null);
                Ok(HashMap::from([("b".into(), a)]))
            }
        }

        let chain = SequentialChain::new(
            vec![Arc::new(AddKeyChain)],
            vec!["a".into()],
            vec!["b".into()],
        );
        let mut input = HashMap::new();
        input.insert("a".into(), Value::String("val".into()));
        let result = chain.call(input).await.unwrap();
        assert_eq!(result.get("b").and_then(|v| v.as_str()), Some("val"));
    }

    #[tokio::test]
    async fn test_sequential_chain_preserves_unused_keys() {
        let chain = SequentialChain::new(
            vec![Arc::new(EchoChain)],
            vec!["input".into()],
            vec!["output".into()],
        );
        let mut input = HashMap::new();
        input.insert("input".into(), Value::String("hi".into()));
        input.insert("extra".into(), Value::String("stay".into()));
        let result = chain.call(input).await.unwrap();
        assert_eq!(result.get("extra").and_then(|v| v.as_str()), Some("stay"));
    }

    #[test]
    fn test_chain_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<SimpleSequentialChain>();
        assert_sync::<SimpleSequentialChain>();
        assert_send::<SequentialChain>();
        assert_sync::<SequentialChain>();
    }
}
