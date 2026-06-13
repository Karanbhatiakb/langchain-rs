//! The core [`LLMChain`] — a runnable chain combining a prompt template, LLM,
//! optional output parser, and optional memory.

use async_trait::async_trait;
use langchain_core::callbacks::CallbackManager;
use langchain_core::errors::*;
use langchain_core::output_parsers::OutputParser;
use langchain_core::prompt::PromptTemplate;
use langchain_core::runnable::Runnable;
use langchain_llms::traits::ChatModel;
use langchain_memory::traits::BaseMemory;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

/// A generic chain that formats a prompt, sends it to an LLM, and optionally
/// parses the output.
///
/// # Type parameters
/// * `T` — The output type (default `String`). Must implement
///   `DeserializeOwned + Serialize + Send`.
pub struct LLMChain<T: DeserializeOwned + Serialize = String> {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    output_parser: Option<Arc<dyn OutputParser<T>>>,
    memory: Option<Arc<dyn BaseMemory>>,
    callbacks: Option<CallbackManager>,
    verbose: bool,
    stop_sequences: Option<Vec<String>>,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize + Send> LLMChain<T> {
    /// Creates a new `LLMChain` with the given LLM and prompt template.
    pub fn new(llm: Arc<dyn ChatModel>, prompt: PromptTemplate) -> Self {
        Self {
            llm,
            prompt,
            output_parser: None,
            memory: None,
            callbacks: None,
            verbose: false,
            stop_sequences: None,
            _phantom: PhantomData,
        }
    }

    /// Sets the output parser (builder pattern).
    pub fn with_output_parser(mut self, parser: Arc<dyn OutputParser<T>>) -> Self {
        self.output_parser = Some(parser);
        self
    }

    /// Sets the memory (builder pattern).
    pub fn with_memory(mut self, memory: Arc<dyn BaseMemory>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Sets the callback manager (builder pattern).
    pub fn with_callbacks(mut self, callbacks: CallbackManager) -> Self {
        self.callbacks = Some(callbacks);
        self
    }

    /// Enables or disables verbose logging (builder pattern).
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Sets the stop sequences (builder pattern).
    pub fn with_stop_sequences(mut self, stops: Vec<String>) -> Self {
        self.stop_sequences = Some(stops);
        self
    }

    /// Formats the prompt, calls the LLM, and parses the response.
    ///
    /// # Errors
    /// Returns [`ChainError`] if prompt formatting, LLM call, or parsing fails.
    pub async fn predict(&self, inputs: &HashMap<String, String>) -> Result<T> {
        let prompt_str = self.prompt.format(inputs)?;

        if self.verbose {
            info!("LLMChain prompt: {}", prompt_str);
        }

        if let Some(ref cb) = self.callbacks {
            cb.on_llm_start("LLMChain", &[prompt_str.clone()]);
        }

        let stop: Option<Vec<&str>> = self
            .stop_sequences
            .as_ref()
            .map(|s| s.iter().map(|x| x.as_str()).collect());

        let response = self
            .llm
            .predict_messages(
                &[langchain_core::messages::HumanMessage::new(&prompt_str).into()],
                None,
                stop.as_deref(),
            )
            .await?;

        if let Some(ref cb) = self.callbacks {
            let val = serde_json::json!({ "response": response.content });
            cb.on_llm_end("LLMChain", &val);
        }

        if let Some(ref parser) = self.output_parser {
            parser.parse(&response.content)
        } else {
            let string_val: String = response.content;
            let val: Value = serde_json::from_str(&format!("\"{}\"", string_val))
                .map_err(|e| ChainError::ParserError(e.to_string()))?;
            let result: T = serde_json::from_value(val)
                .map_err(|e| ChainError::ParserError(e.to_string()))?;
            Ok(result)
        }
    }
}

#[async_trait]
impl<T: DeserializeOwned + Serialize + Send + Sync> Runnable<HashMap<String, Value>, Value>
    for LLMChain<T>
{
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<Value> {
        let result = self.call(input).await?;
        Ok(result
            .into_values()
            .next()
            .unwrap_or(Value::Null))
    }

    async fn stream(
        &self,
        _input: HashMap<String, Value>,
    ) -> Result<futures::stream::BoxStream<'static, Result<Value>>> {
        Err(ChainError::LLMError("Streaming not supported for LLMChain".into()))
    }
}

#[async_trait]
impl<T: DeserializeOwned + Serialize + Send + Sync> Chain for LLMChain<T> {
    fn input_keys(&self) -> Vec<String> {
        self.prompt.input_variables.clone()
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let mut resolved_inputs = HashMap::new();

        if let Some(ref memory) = self.memory {
            let memory_vars = memory.load_memory_variables(&inputs).await?;
            for (k, v) in memory_vars {
                let str_val = v.as_str().map(|s| s.to_string()).unwrap_or_default();
                resolved_inputs.insert(k, str_val);
            }
        }

        for (k, v) in &inputs {
            let str_val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
            resolved_inputs.insert(k.clone(), str_val);
        }

        if self.verbose {
            info!("LLMChain.call with inputs: {:?}", resolved_inputs);
        }

        if let Some(ref cb) = self.callbacks {
            let json = serde_json::to_value(&resolved_inputs).unwrap_or_default();
            cb.on_chain_start("LLMChain", &json);
        }

        let prompt_str = self.prompt.format(&resolved_inputs)?;

        let stop: Option<Vec<&str>> = self
            .stop_sequences
            .as_ref()
            .map(|s| s.iter().map(|x| x.as_str()).collect());

        let response = self
            .llm
            .predict_messages(
                &[langchain_core::messages::HumanMessage::new(&prompt_str).into()],
                None,
                stop.as_deref(),
            )
            .await?;

        let output_str = if let Some(ref parser) = self.output_parser {
            let parsed = parser.parse(&response.content)?;
            serde_json::to_value(&parsed)
                .map_err(|e| ChainError::ParserError(e.to_string()))?
        } else {
            Value::String(response.content)
        };

        let mut result = HashMap::new();
        result.insert("output".to_string(), output_str);

        if let Some(ref memory) = self.memory {
            memory.save_context(&inputs, &result).await?;
        }

        if let Some(ref cb) = self.callbacks {
            let json = serde_json::to_value(&result).unwrap_or_default();
            cb.on_chain_end("LLMChain", &json);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use langchain_core::prompt::PromptTemplate;
    use langchain_llms::fake::FakeListLLM;

    #[tokio::test]
    async fn test_llm_chain_new() {
        let llm = Arc::new(FakeListLLM::new(vec!["Hello!".into()]));
        let prompt = PromptTemplate::from_template("Say: {input}");
        let chain = LLMChain::<String>::new(llm, prompt);
        assert_eq!(chain.input_keys(), vec!["input"]);
        assert_eq!(chain.output_keys(), vec!["output"]);
    }

    #[tokio::test]
    async fn test_llm_chain_predict() {
        let llm = Arc::new(FakeListLLM::new(vec!["Mock response".into()]));
        let prompt = PromptTemplate::from_template("Say: {input}");
        let chain = LLMChain::<String>::new(llm, prompt);

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), "hello".into());
        let result = chain.predict(&inputs).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_llm_chain_call() {
        let llm = Arc::new(FakeListLLM::new(vec!["Mock answer".into()]));
        let prompt = PromptTemplate::from_template("Q: {question}");
        let chain = LLMChain::<String>::new(llm, prompt);

        let mut inputs = HashMap::new();
        inputs.insert("question".into(), Value::String("What?".into()));
        let result = chain.call(inputs).await.unwrap();
        assert!(result.contains_key("output"));
    }

    #[tokio::test]
    async fn test_llm_chain_with_verbose() {
        let llm = Arc::new(FakeListLLM::new(vec!["Verbose".into()]));
        let prompt = PromptTemplate::from_template("{input}");
        let chain = LLMChain::<String>::new(llm, prompt).with_verbose(true);

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), "test".into());
        let result = chain.predict(&inputs).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_llm_chain_with_stop_sequences() {
        let llm = Arc::new(FakeListLLM::new(vec!["Stopped".into()]));
        let prompt = PromptTemplate::from_template("{input}");
        let chain = LLMChain::<String>::new(llm, prompt)
            .with_stop_sequences(vec!["stop".into()]);

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("go".into()));
        let result = chain.call(inputs).await.unwrap();
        assert!(result.contains_key("output"));
    }

    #[tokio::test]
    async fn test_llm_chain_stream_returns_error() {
        let llm = Arc::new(FakeListLLM::new(vec!["test".into()]));
        let prompt = PromptTemplate::from_template("{input}");
        let chain = LLMChain::<String>::new(llm, prompt);

        let mut inputs = HashMap::new();
        inputs.insert("input".into(), Value::String("x".into()));
        let result = chain.stream(inputs).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_llm_chain_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        let llm = Arc::new(FakeListLLM::new(vec!["test".into()]));
        let prompt = PromptTemplate::from_template("{input}");
        let _chain = LLMChain::<String>::new(llm, prompt);
        assert_send::<LLMChain<String>>();
        assert_sync::<LLMChain<String>>();
    }
}
