//! Core `Runnable` trait that underpins the LangChain Expression Language (LCEL).
//!
//! Every invocable component (LLM, chain, retriever, tool, etc.) implements
//! [`Runnable`], providing a uniform interface for invoke, batch, stream, and
//! transform operations.

use crate::errors::*;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Core trait for LangChain components that can be invoked with an input and
/// produce an output.
///
/// Provides default implementations for `batch`, `stream`, and `transform` —
/// override them when the underlying provider supports those operations
/// natively.
///
/// # Type parameters
/// * `I` — Input type (must be `Send + 'static`).
/// * `O` — Output type (must be `Send + 'static`).
#[async_trait]
pub trait Runnable<I: Send + 'static, O: Send + 'static>: Send + Sync {
    /// Invoke the runnable with a single input and return the output.
    async fn invoke(&self, input: I) -> Result<O>;

    /// Invoke the runnable on a batch of inputs, returning a vector of
    /// outputs in the same order.
    ///
    /// The default implementation calls `invoke` sequentially.
    async fn batch(&self, inputs: Vec<I>) -> Result<Vec<O>> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            results.push(self.invoke(input).await?);
        }
        Ok(results)
    }

    /// Stream output chunks from the runnable.
    ///
    /// The default implementation returns an error indicating streaming is not
    /// supported.
    async fn stream(&self, _input: I) -> Result<BoxStream<'static, Result<O>>> {
        Err(ChainError::LLMError("Streaming not supported".into()))
    }

    /// Transform an input stream of items into an output stream.
    ///
    /// The default implementation buffers the input stream, invokes each item
    /// sequentially, and yields the results.
    async fn transform(
        &self,
        input: BoxStream<'static, Result<I>>,
    ) -> Result<BoxStream<'static, Result<O>>> {
        use futures::StreamExt;
        let mut input = input;
        let mut results = Vec::new();
        while let Some(item) = input.next().await {
            let item = item?;
            results.push(self.invoke(item).await?);
        }
        let stream = futures::stream::iter(results.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }
}

/// Erased, cloneable function pointer wrapping a `Runnable<I, O>::invoke`.
///
/// Used internally by LCEL combinators to store runnables without
/// monomorphising over the concrete type.
pub type DynRunnableFn<I, O> = Arc<
    dyn Fn(I) -> Pin<Box<dyn Future<Output = Result<O>> + Send>> + Send + Sync,
>;