//! Example selectors for choosing which few-shot examples to include in a
//! prompt.

use crate::errors::*;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub mod length_based;
pub mod ngram;
pub mod maximal_margin;

pub use length_based::LengthBasedExampleSelector;
pub use ngram::NGramExampleSelector;
pub use maximal_margin::MaximalMarginalRelevanceExampleSelector;

pub fn get_example_text_length(example: &HashMap<String, Value>) -> usize {
    example
        .values()
        .map(|v| match v {
            Value::String(s) => s.len(),
            _ => v.to_string().len(),
        })
        .sum()
}

#[async_trait]
pub trait ExampleSelector: Send + Sync {
    async fn select_examples(
        &self,
        input_variables: &HashMap<String, Value>,
    ) -> Result<Vec<HashMap<String, Value>>>;
}
