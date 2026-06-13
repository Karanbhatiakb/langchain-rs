//! RunnableBranch — dispatches to different runnables based on conditions.

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type ConditionFn = Arc<
    dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>>
        + Send
        + Sync,
>;

type BranchFn = Arc<
    dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>>> + Send>>
        + Send
        + Sync,
>;

/// A runnable that evaluates conditions in order and invokes the first
/// matching branch's runnable. If no condition matches, an optional default
/// runnable is used; otherwise an error is returned.
pub struct RunnableBranch {
    branches: Vec<(ConditionFn, BranchFn)>,
    default: Option<BranchFn>,
}

impl RunnableBranch {
    /// Creates a new `RunnableBranch` with no branches.
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            default: None,
        }
    }

    /// Adds a branch with a condition function and a runnable.
    ///
    /// The condition function receives the input dict and returns `Ok(true)`
    /// if this branch should handle the input.
    pub fn add_branch<F, Fut, R, Fut2>(mut self, condition_fn: F, runnable: R) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<bool>> + Send + 'static,
        R: Fn(HashMap<String, Value>) -> Fut2 + Send + Sync + 'static,
        Fut2: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        let cond: ConditionFn =
            Arc::new(move |input: HashMap<String, Value>| Box::pin(condition_fn(input)));
        let run: BranchFn =
            Arc::new(move |input: HashMap<String, Value>| Box::pin(runnable(input)));
        self.branches.push((cond, run));
        self
    }

    /// Sets a default runnable that is invoked when no branch condition matches.
    pub fn with_default<F, Fut>(mut self, runnable: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        self.default = Some(Arc::new(move |input: HashMap<String, Value>| {
            Box::pin(runnable(input))
        }));
        self
    }
}

impl Default for RunnableBranch {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RunnableBranch {
    fn clone(&self) -> Self {
        Self {
            branches: self.branches.clone(),
            default: self.default.clone(),
        }
    }
}

impl std::fmt::Debug for RunnableBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunnableBranch")
            .field("branches", &self.branches.len())
            .field("default", &self.default.is_some())
            .finish()
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnableBranch {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        for (condition, runnable) in &self.branches {
            if condition(input.clone()).await? {
                return runnable(input).await;
            }
        }
        match &self.default {
            Some(default_fn) => default_fn(input).await,
            None => Err(ChainError::ValidationError(
                "No branch condition matched and no default runnable set".into(),
            )),
        }
    }
}
