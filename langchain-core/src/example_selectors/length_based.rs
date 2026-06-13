//! Length-based example selector with budget concept from Python.

use crate::errors::*;
use crate::example_selectors::{ExampleSelector, get_example_text_length};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub struct LengthBasedExampleSelector {
    pub examples: Vec<HashMap<String, Value>>,
    pub max_length: usize,
    pub get_length: Box<dyn Fn(&HashMap<String, Value>) -> usize + Send + Sync>,
}

impl LengthBasedExampleSelector {
    pub fn new(examples: Vec<HashMap<String, Value>>, max_length: usize) -> Self {
        Self {
            examples,
            max_length,
            get_length: Box::new(get_example_text_length),
        }
    }

    pub fn with_length_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&HashMap<String, Value>) -> usize + Send + Sync + 'static,
    {
        self.get_length = Box::new(f);
        self
    }

    pub fn add_example(&mut self, example: HashMap<String, Value>) {
        self.examples.push(example);
    }

    fn calculate_example_length(&self, example: &HashMap<String, Value>) -> usize {
        (self.get_length)(example)
    }
}

#[async_trait]
impl ExampleSelector for LengthBasedExampleSelector {
    async fn select_examples(
        &self,
        input_variables: &HashMap<String, Value>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        let input_text = input_variables
            .values()
            .map(|v| match v {
                Value::String(s) => s.len(),
                _ => v.to_string().len(),
            })
            .sum::<usize>();

        let input_budget = (input_text as f64 * 0.2) as usize;
        let remaining_budget = self.max_length.saturating_sub(input_budget);

        let mut selected = Vec::new();
        let mut total_length = 0usize;

        for example in &self.examples {
            let example_length = self.calculate_example_length(example);
            if total_length + example_length <= remaining_budget {
                selected.push(example.clone());
                total_length += example_length;
            } else {
                break;
            }
        }

        Ok(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_example(text: &str) -> HashMap<String, Value> {
        HashMap::from([("text".to_string(), Value::String(text.to_string()))])
    }

    #[tokio::test]
    async fn test_length_based_selects_all_within_budget() {
        let examples = vec![
            make_example("short"),
            make_example("tiny"),
        ];
        let selector = LengthBasedExampleSelector::new(examples, 1000);
        let input = HashMap::from([("q".to_string(), Value::String("hi".to_string()))]);
        let selected = selector.select_examples(&input).await.unwrap();
        assert_eq!(selected.len(), 2);
    }

    #[tokio::test]
    async fn test_length_based_respects_max_length() {
        let examples = vec![
            make_example(&"a".repeat(100)),
            make_example(&"b".repeat(100)),
            make_example(&"c".repeat(100)),
        ];
        let selector = LengthBasedExampleSelector::new(examples, 50);
        let input = HashMap::from([("q".to_string(), Value::String("x".to_string()))]);
        let selected = selector.select_examples(&input).await.unwrap();
        assert!(selected.len() <= 1);
    }

    #[tokio::test]
    async fn test_length_based_add_example() {
        let mut selector = LengthBasedExampleSelector::new(vec![], 100);
        selector.add_example(make_example("new"));
        assert_eq!(selector.examples.len(), 1);
    }

    #[tokio::test]
    async fn test_length_based_empty_examples() {
        let selector = LengthBasedExampleSelector::new(vec![], 100);
        let input = HashMap::from([("q".to_string(), Value::String("test".to_string()))]);
        let selected = selector.select_examples(&input).await.unwrap();
        assert!(selected.is_empty());
    }

    #[tokio::test]
    async fn test_length_based_with_custom_length_fn() {
        let examples = vec![make_example("hello")];
        let selector = LengthBasedExampleSelector::new(examples, 100)
            .with_length_fn(|_| 200);
        let input = HashMap::from([("q".to_string(), Value::String("test".to_string()))]);
        let selected = selector.select_examples(&input).await.unwrap();
        assert!(selected.is_empty());
    }

    #[tokio::test]
    async fn test_get_example_text_length_string() {
        let example = HashMap::from([("a".to_string(), Value::String("hello".to_string()))]);
        assert_eq!(crate::example_selectors::get_example_text_length(&example), 5);
    }

    #[tokio::test]
    async fn test_get_example_text_length_number() {
        let example = HashMap::from([("n".to_string(), Value::Number(serde_json::Number::from(42)))]);
        assert!(crate::example_selectors::get_example_text_length(&example) > 0);
    }
}
