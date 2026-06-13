//! lcel_demo module.

use langchain_core::errors::*;
use langchain_core::runnable::Runnable;
use serde_json::Value;

struct PassthroughRunnable;

#[async_trait::async_trait]
impl Runnable<Value, Value> for PassthroughRunnable {
    async fn invoke(&self, input: Value) -> Result<Value> {
        Ok(input)
    }
}

struct UpperCaseRunnable;

#[async_trait::async_trait]
impl Runnable<Value, Value> for UpperCaseRunnable {
    async fn invoke(&self, input: Value) -> Result<Value> {
        match input {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            other => Ok(other),
        }
    }
}

struct ExclaimRunnable;

#[async_trait::async_trait]
impl Runnable<Value, Value> for ExclaimRunnable {
    async fn invoke(&self, input: Value) -> Result<Value> {
        match input {
            Value::String(s) => Ok(Value::String(format!("{}!", s))),
            other => Ok(other),
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    let passthrough = PassthroughRunnable;
    let input = Value::String("hello world".to_string());
    let result = passthrough.invoke(input).await?;
    println!("RunnablePassthrough: {}", result);

    let upper = UpperCaseRunnable;
    let exclaim = ExclaimRunnable;
    let input = Value::String("hello".to_string());
    let upper_result = upper.invoke(input.clone()).await?;
    let final_result = exclaim.invoke(upper_result).await?;
    println!("RunnableSequence (upper + exclaim): {}", final_result);

    let input = Value::String("rust".to_string());
    let upper_chain = UpperCaseRunnable;
    let result = upper_chain.invoke(input).await?;
    println!("UpperCaseRunnable: {}", result);

    let input = Value::String("HELLO".to_string());
    let exclaim_chain = ExclaimRunnable;
    let result = exclaim_chain.invoke(input).await?;
    println!("ExclaimRunnable: {}", result);

    Ok(())
}
