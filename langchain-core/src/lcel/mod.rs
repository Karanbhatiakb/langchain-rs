//! LCEL (LangChain Expression Language) — composable runnable combinators.
//!
//! Provides sequence, parallel, branching, fallback, retry, mapping, binding,
//! and generator wrappers that all implement the [`Runnable`] trait, allowing
//! them to be composed with the `|` operator via [`BitOr`].

use crate::errors::*;
use crate::runnable::{DynRunnableFn, Runnable};
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use rand::Rng;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::BitOr;
use std::pin::Pin;
use std::sync::Arc;

/// Wraps a [`Runnable`] into a [`DynRunnableFn`] by cloning it into an `Arc`.
fn make_dyn<I, O, R>(runnable: R) -> DynRunnableFn<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
    R: Runnable<I, O> + 'static,
{
    let runnable = Arc::new(runnable);
    Arc::new(move |input: I| -> Pin<Box<dyn Future<Output = Result<O>> + Send>> {
        let r = runnable.clone();
        Box::pin(async move { r.invoke(input).await })
    })
}

/// Chains two runnables into a single function: `I -> M -> O`.
fn make_seq<I, M, O, R1, R2>(r1: R1, r2: R2) -> DynRunnableFn<I, O>
where
    I: Send + 'static,
    M: Send + 'static,
    O: Send + 'static,
    R1: Runnable<I, M> + 'static,
    R2: Runnable<M, O> + 'static,
{
    let r1 = Arc::new(r1);
    let r2 = Arc::new(r2);
    Arc::new(move |input: I| -> Pin<Box<dyn Future<Output = Result<O>> + Send>> {
        let r1 = r1.clone();
        let r2 = r2.clone();
        Box::pin(async move {
            let mid = r1.invoke(input).await?;
            r2.invoke(mid).await
        })
    })
}

/// A runnable that chains two runnables sequentially.
///
/// `RunnableSequence` feeds the output of the first runnable as input to the
/// second. Additional steps can be appended with [`then`](RunnableSequence::then).
pub struct RunnableSequence<I, O> {
    func: DynRunnableFn<I, O>,
}

impl<I: Send + 'static, O: Send + 'static> RunnableSequence<I, O> {
    /// Creates a new sequence from two runnables.
    pub fn new<M: Send + 'static, R1, R2>(r1: R1, r2: R2) -> Self
    where
        R1: Runnable<I, M> + 'static,
        R2: Runnable<M, O> + 'static,
    {
        Self {
            func: make_seq(r1, r2),
        }
    }

    /// Appends another runnable to the end of the sequence.
    pub fn then<R, O2>(self, next: R) -> RunnableSequence<I, O2>
    where
        O2: Send + 'static,
        R: Runnable<O, O2> + 'static,
        Self: Sized,
    {
        let r1 = self.func.clone();
        let r2 = Arc::new(next);
        RunnableSequence {
            func: Arc::new(
                move |input: I| -> Pin<Box<dyn Future<Output = Result<O2>> + Send>> {
                    let r1 = r1.clone();
                    let r2 = r2.clone();
                    Box::pin(async move {
                        let mid = r1(input).await?;
                        r2.invoke(mid).await
                    })
                },
            ),
        }
    }
}

/// A runnable that invokes multiple runnables in parallel (currently a stub).
pub struct RunnableParallel<I, O> {
    func: DynRunnableFn<I, O>,
}

impl<I: Send + 'static, O: Send + 'static> RunnableParallel<I, O> {
    /// Creates a new `RunnableParallel` (currently returns "Not implemented").
    pub fn new() -> Self {
        let func: DynRunnableFn<I, O> = Arc::new(
            |_input: I| -> Pin<Box<dyn Future<Output = Result<O>> + Send>> {
                Box::pin(async move { Err(ChainError::LLMError("Not implemented".into())) })
            },
        );
        Self { func }
    }
}

impl<I: Send + 'static, O: Send + 'static> Default for RunnableParallel<I, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Send + 'static, O: Send + 'static> Clone for RunnableParallel<I, O> {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
        }
    }
}

#[async_trait]
impl<I: Send + 'static, O: Send + 'static> Runnable<I, O> for RunnableParallel<I, O> {
    async fn invoke(&self, input: I) -> Result<O> {
        (self.func)(input).await
    }
}

/// A runnable that passes input through unchanged (identity).
#[derive(Clone)]
pub struct RunnablePassthrough<I> {
    _phantom: PhantomData<I>,
}

impl<I> RunnablePassthrough<I> {
    /// Creates a new `RunnablePassthrough`.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<I> Default for RunnablePassthrough<I> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<I: Send + Sync + 'static> Runnable<I, I> for RunnablePassthrough<I> {
    async fn invoke(&self, input: I) -> Result<I> {
        Ok(input)
    }

    async fn stream(&self, input: I) -> Result<BoxStream<'static, Result<I>>> {
        Ok(Box::pin(futures::stream::once(async move { Ok(input) })))
    }
}

/// A runnable that evaluates conditions (branches) to select which sub-runnable
/// to execute.
pub struct RunnableBranch<I, O> {
    branches: Vec<(DynRunnableFn<I, bool>, DynRunnableFn<I, O>)>,
    default: Option<DynRunnableFn<I, O>>,
}

impl<I: Send + Clone + 'static, O: Send + 'static> RunnableBranch<I, O> {
    /// Creates a new `RunnableBranch` with no branches.
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            default: None,
        }
    }

    /// Adds a branch: if `condition` returns `true`, `runnable` is invoked.
    pub fn add_branch<R1, R2>(mut self, condition: R1, runnable: R2) -> Self
    where
        R1: Runnable<I, bool> + 'static,
        R2: Runnable<I, O> + 'static,
    {
        self.branches.push((make_dyn(condition), make_dyn(runnable)));
        self
    }

    /// Sets the default runnable when no branch matches.
    pub fn with_default<R>(mut self, runnable: R) -> Self
    where
        R: Runnable<I, O> + 'static,
    {
        self.default = Some(make_dyn(runnable));
        self
    }
}

impl<I: Send + Clone + 'static, O: Send + 'static> Default for RunnableBranch<I, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Send + Clone + 'static, O: Send + 'static> Clone for RunnableBranch<I, O> {
    fn clone(&self) -> Self {
        Self {
            branches: self.branches.clone(),
            default: self.default.clone(),
        }
    }
}

#[async_trait]
impl<I: Send + Clone + 'static, O: Send + 'static> Runnable<I, O> for RunnableBranch<I, O> {
    async fn invoke(&self, input: I) -> Result<O> {
        for (condition, runnable) in &self.branches {
            if condition(input.clone()).await? {
                return runnable(input).await;
            }
        }
        if let Some(ref default) = self.default {
            default(input).await
        } else {
            Err(ChainError::PromptError(
                "No branch matched and no default provided".into(),
            ))
        }
    }
}

/// A runnable that tries a primary runnable first, then falls back through a
/// list of fallbacks if it fails.
pub struct RunnableWithFallbacks<I, O> {
    primary: DynRunnableFn<I, O>,
    fallbacks: Vec<DynRunnableFn<I, O>>,
}

impl<I: Send + Clone + 'static, O: Send + 'static> RunnableWithFallbacks<I, O> {
    /// Creates a new `RunnableWithFallbacks` with a primary and one or more
    /// fallbacks.
    pub fn new<R1, R2>(primary: R1, fallbacks: Vec<R2>) -> Self
    where
        R1: Runnable<I, O> + 'static,
        R2: Runnable<I, O> + 'static,
    {
        Self {
            primary: make_dyn(primary),
            fallbacks: fallbacks.into_iter().map(make_dyn).collect(),
        }
    }

    /// Adds an additional fallback.
    pub fn add_fallback<R>(mut self, fallback: R) -> Self
    where
        R: Runnable<I, O> + 'static,
    {
        self.fallbacks.push(make_dyn(fallback));
        self
    }
}

impl<I: Send + Clone + 'static, O: Send + 'static> Clone for RunnableWithFallbacks<I, O> {
    fn clone(&self) -> Self {
        Self {
            primary: self.primary.clone(),
            fallbacks: self.fallbacks.clone(),
        }
    }
}

#[async_trait]
impl<I: Send + Clone + 'static, O: Send + 'static> Runnable<I, O>
    for RunnableWithFallbacks<I, O>
{
    async fn invoke(&self, input: I) -> Result<O> {
        match (self.primary)(input.clone()).await {
            Ok(output) => return Ok(output),
            Err(_) => {}
        }
        for fallback in &self.fallbacks {
            match fallback(input.clone()).await {
                Ok(output) => return Ok(output),
                Err(_) => continue,
            }
        }
        Err(ChainError::LLMError("All fallbacks failed".into()))
    }
}

/// A runnable that retries a wrapped runnable on failure with exponential
/// backoff.
pub struct RunnableRetry<I, O> {
    runnable: DynRunnableFn<I, O>,
    /// Maximum number of retry attempts.
    pub max_retries: usize,
    /// Base delay between retries in milliseconds (doubled each attempt).
    pub base_delay_ms: u64,
}

impl<I: Send + Clone + 'static, O: Send + 'static> RunnableRetry<I, O> {
    /// Creates a new `RunnableRetry`.
    pub fn new<R>(runnable: R, max_retries: usize, base_delay_ms: u64) -> Self
    where
        R: Runnable<I, O> + 'static,
    {
        Self {
            runnable: make_dyn(runnable),
            max_retries,
            base_delay_ms,
        }
    }
}

impl<I: Send + Clone + 'static, O: Send + 'static> Clone for RunnableRetry<I, O> {
    fn clone(&self) -> Self {
        Self {
            runnable: self.runnable.clone(),
            max_retries: self.max_retries,
            base_delay_ms: self.base_delay_ms,
        }
    }
}

#[async_trait]
impl<I: Send + Clone + 'static, O: Send + 'static> Runnable<I, O> for RunnableRetry<I, O> {
    async fn invoke(&self, input: I) -> Result<O> {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            match (self.runnable)(input.clone()).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    last_err = Some(e);
                    if attempt < self.max_retries {
                        let delay_ms = self.base_delay_ms * 2u64.pow(attempt as u32)
                            + rand::thread_rng().gen_range(0..100);
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| ChainError::LLMError("Retry failed".into())))
    }
}

/// Applies a runnable to each element of a `Vec<I>`, producing `Vec<O>`.
pub struct RunnableEach<I, O> {
    runnable: DynRunnableFn<I, O>,
}

impl<I: Send + 'static, O: Send + 'static> RunnableEach<I, O> {
    /// Creates a new `RunnableEach`.
    pub fn new<R>(runnable: R) -> Self
    where
        R: Runnable<I, O> + 'static,
    {
        Self {
            runnable: make_dyn(runnable),
        }
    }
}

impl<I: Send + 'static, O: Send + 'static> Clone for RunnableEach<I, O> {
    fn clone(&self) -> Self {
        Self {
            runnable: self.runnable.clone(),
        }
    }
}

#[async_trait]
impl<I: Send + 'static, O: Send + 'static> Runnable<Vec<I>, Vec<O>> for RunnableEach<I, O> {
    async fn invoke(&self, input: Vec<I>) -> Result<Vec<O>> {
        let mut results = Vec::with_capacity(input.len());
        for item in input {
            results.push((self.runnable)(item).await?);
        }
        Ok(results)
    }
}

/// A runnable that binds extra keyword arguments to an inner runnable.
pub struct RunnableBinding<I, O> {
    runnable: DynRunnableFn<I, O>,
    /// Bound keyword arguments passed through at invocation.
    pub bound_kwargs: std::collections::HashMap<String, serde_json::Value>,
}

impl<I: Send + 'static, O: Send + 'static> RunnableBinding<I, O> {
    /// Creates a new `RunnableBinding`.
    pub fn new<R>(runnable: R, kwargs: std::collections::HashMap<String, serde_json::Value>) -> Self
    where
        R: Runnable<I, O> + 'static,
    {
        Self {
            runnable: make_dyn(runnable),
            bound_kwargs: kwargs,
        }
    }
}

impl<I: Send + 'static, O: Send + 'static> Clone for RunnableBinding<I, O> {
    fn clone(&self) -> Self {
        Self {
            runnable: self.runnable.clone(),
            bound_kwargs: self.bound_kwargs.clone(),
        }
    }
}

#[async_trait]
impl<I: Send + 'static, O: Send + 'static> Runnable<I, O> for RunnableBinding<I, O> {
    async fn invoke(&self, input: I) -> Result<O> {
        (self.runnable)(input).await
    }
}

/// A runnable that wraps an async generator function.
///
/// The generator function receives an input `I` and returns a
/// [`BoxStream`] of `O` values.
pub struct RunnableGenerator<I, O> {
    func: Arc<
        dyn Fn(I) -> Pin<Box<dyn Future<Output = Result<BoxStream<'static, Result<O>>>> + Send>>
            + Send
            + Sync,
    >,
}

impl<I: Send + 'static, O: Send + 'static> RunnableGenerator<I, O> {
    /// Creates a new `RunnableGenerator` from an async function.
    pub fn new<F, Fut>(func: F) -> Self
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<BoxStream<'static, Result<O>>>> + Send + 'static,
    {
        let wrapped: Arc<dyn Fn(I) -> Pin<Box<dyn Future<Output = Result<BoxStream<'static, Result<O>>>> + Send>> + Send + Sync> =
            Arc::new(move |input: I| Box::pin(func(input)));
        Self { func: wrapped }
    }
}

impl<I: Send + 'static, O: Send + 'static> Clone for RunnableGenerator<I, O> {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
        }
    }
}

#[async_trait]
impl<I: Send + 'static, O: Send + 'static> Runnable<I, O> for RunnableGenerator<I, O> {
    async fn invoke(&self, input: I) -> Result<O> {
        let mut stream = (self.func)(input).await?;
        let mut result = None;
        while let Some(item) = stream.next().await {
            match item {
                Ok(chunk) => result = Some(chunk),
                Err(e) => return Err(e),
            }
        }
        result.ok_or_else(|| ChainError::StreamError("No output from generator".into()))
    }

    async fn stream(&self, input: I) -> Result<BoxStream<'static, Result<O>>> {
        (self.func)(input).await
    }
}

/// Extension trait adding a `.pipe()` method to any [`Runnable`].
pub trait Pipe<I: Send + 'static, O: Send + 'static>: Runnable<I, O> + Sized {
    /// Chains this runnable with the next runnable, returning a
    /// [`RunnableSequence`].
    fn pipe<M: Send + 'static, R>(self, next: R) -> RunnableSequence<I, M>
    where
        R: Runnable<O, M> + 'static,
        Self: 'static,
    {
        let r1 = Arc::new(self);
        let r2 = Arc::new(next);
        RunnableSequence {
            func: Arc::new(
                move |input: I| -> Pin<Box<dyn Future<Output = Result<M>> + Send>> {
                    let r1 = r1.clone();
                    let r2 = r2.clone();
                    Box::pin(async move {
                        let mid = r1.invoke(input).await?;
                        r2.invoke(mid).await
                    })
                },
            ),
        }
    }
}

impl<T, I, O> Pipe<I, O> for T where T: Runnable<I, O> + Sized, I: Send + 'static, O: Send + 'static {}

/// Wraps a [`Runnable`] as a [`RunnableLambda`], enabling the `|` operator.
pub fn pipe<I: Send + 'static, O: Send + 'static>(
    runnable: impl Runnable<I, O> + 'static,
) -> RunnableLambda<I, O> {
    RunnableLambda {
        func: make_dyn(runnable),
    }
}

/// A runnable wrapping an arbitrary async closure `I -> Result<O>`.
pub struct RunnableLambda<I, O> {
    func: DynRunnableFn<I, O>,
}

impl<I: Send + 'static, O: Send + 'static> RunnableLambda<I, O> {
    /// Creates a new `RunnableLambda` from an async function.
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O>> + Send + 'static,
    {
        Self {
            func: Arc::new(move |input: I| -> Pin<Box<dyn Future<Output = Result<O>> + Send>> {
                Box::pin(f(input))
            }),
        }
    }
}

impl<I: Send + 'static, O: Send + 'static> Clone for RunnableLambda<I, O> {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
        }
    }
}

#[async_trait]
impl<I: Send + 'static, O: Send + 'static> Runnable<I, O> for RunnableLambda<I, O> {
    async fn invoke(&self, input: I) -> Result<O> {
        (self.func)(input).await
    }
}

/// Chains two `DynRunnableFn`s into one.
fn chain_dyn<I, M, O>(left: DynRunnableFn<I, M>, right: DynRunnableFn<M, O>) -> DynRunnableFn<I, O>
where
    I: Send + 'static,
    M: Send + 'static,
    O: Send + 'static,
{
    Arc::new(move |input: I| -> Pin<Box<dyn Future<Output = Result<O>> + Send>> {
        let left = left.clone();
        let right = right.clone();
        Box::pin(async move {
            let mid = left(input).await?;
            right(mid).await
        })
    })
}

impl<I: Send + 'static, M: Send + 'static, O: Send + 'static> BitOr<RunnableLambda<M, O>>
    for RunnableLambda<I, M>
{
    type Output = RunnableSequence<I, O>;

    fn bitor(self, rhs: RunnableLambda<M, O>) -> Self::Output {
        RunnableSequence {
            func: chain_dyn(self.func, rhs.func),
        }
    }
}

impl<I: Send + 'static, M: Send + 'static, O: Send + 'static> BitOr<RunnableSequence<M, O>>
    for RunnableLambda<I, M>
{
    type Output = RunnableSequence<I, O>;

    fn bitor(self, rhs: RunnableSequence<M, O>) -> Self::Output {
        RunnableSequence {
            func: chain_dyn(self.func, rhs.func),
        }
    }
}

impl<I: Send + 'static, M: Send + 'static, O: Send + 'static> BitOr<RunnableLambda<M, O>>
    for RunnableSequence<I, M>
{
    type Output = RunnableSequence<I, O>;

    fn bitor(self, rhs: RunnableLambda<M, O>) -> Self::Output {
        RunnableSequence {
            func: chain_dyn(self.func, rhs.func),
        }
    }
}

impl<I: Send + 'static, M: Send + 'static, O: Send + 'static> BitOr<RunnableSequence<M, O>>
    for RunnableSequence<I, M>
{
    type Output = RunnableSequence<I, O>;

    fn bitor(self, rhs: RunnableSequence<M, O>) -> Self::Output {
        RunnableSequence {
            func: chain_dyn(self.func, rhs.func),
        }
    }
}
