//! RunnableRouter — routes to different runnables by key.

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type RouteFn = Arc<
    dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<HashMap<String, Value>>> + Send>>
        + Send
        + Sync,
>;

/// A runnable that reads a route key from the input dict and dispatches
/// to the corresponding registered runnable.
///
/// If the key is missing or no route matches, an optional default runnable
/// is used; otherwise an error is returned.
pub struct RunnableRouter {
    routes: HashMap<String, RouteFn>,
    default: Option<RouteFn>,
    /// The key in the input dict whose value determines the route.
    pub route_key: String,
}

impl RunnableRouter {
    /// Creates a new `RunnableRouter` that reads the route from the given key.
    pub fn new(route_key: impl Into<String>) -> Self {
        Self {
            routes: HashMap::new(),
            default: None,
            route_key: route_key.into(),
        }
    }

    /// Registers a runnable for a specific route key value.
    pub fn add_route<F, Fut>(mut self, key: impl Into<String>, runnable: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HashMap<String, Value>>> + Send + 'static,
    {
        let route_fn: RouteFn =
            Arc::new(move |input: HashMap<String, Value>| Box::pin(runnable(input)));
        self.routes.insert(key.into(), route_fn);
        self
    }

    /// Sets a default runnable used when no route matches the input key.
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

impl Clone for RunnableRouter {
    fn clone(&self) -> Self {
        Self {
            routes: self.routes.clone(),
            default: self.default.clone(),
            route_key: self.route_key.clone(),
        }
    }
}

impl std::fmt::Debug for RunnableRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunnableRouter")
            .field("route_key", &self.route_key)
            .field("routes", &self.routes.len())
            .field("default", &self.default.is_some())
            .finish()
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnableRouter {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let route_value = input.get(&self.route_key).ok_or_else(|| {
            ChainError::ValidationError(format!(
                "Route key '{}' not found in input",
                self.route_key
            ))
        })?;

        let route_str = match route_value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        match self.routes.get(&route_str) {
            Some(runnable) => runnable(input).await,
            None => match &self.default {
                Some(default_fn) => default_fn(input).await,
                None => Err(ChainError::ValidationError(format!(
                    "No route found for key '{}' and no default runnable set",
                    route_str
                ))),
            },
        }
    }
}
