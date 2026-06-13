//! RunnablePassthrough — passes input through unchanged.

use crate::errors::*;
use crate::runnable::Runnable;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// A runnable that returns its input unchanged.
///
/// Useful as a no-op placeholder in LCEL pipelines or to explicitly
/// indicate that a particular step performs no transformation.
#[derive(Debug, Clone, Default)]
pub struct RunnablePassthrough;

impl RunnablePassthrough {
    /// Creates a new `RunnablePassthrough`.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runnable<HashMap<String, Value>, HashMap<String, Value>> for RunnablePassthrough {
    async fn invoke(&self, input: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        Ok(input)
    }
}
