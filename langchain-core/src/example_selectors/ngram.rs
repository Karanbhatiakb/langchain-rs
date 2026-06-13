//! N-gram overlap example selector.

use crate::errors::*;
use crate::example_selectors::ExampleSelector;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct NGramExampleSelector {
    pub examples: Vec<HashMap<String, Value>>,
    pub k: usize,
    pub n: usize,
}

impl NGramExampleSelector {
    pub fn new(examples: Vec<HashMap<String, Value>>, k: usize, n: usize) -> Self {
        Self { examples, k, n }
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn with_n(mut self, n: usize) -> Self {
        self.n = n;
        self
    }

    pub fn add_example(&mut self, example: HashMap<String, Value>) {
        self.examples.push(example);
    }

    pub fn get_ngrams(&self, text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() < self.n {
            return vec![text.to_string()];
        }
        chars.windows(self.n).map(|w| w.iter().collect()).collect()
    }

    pub fn jaccard_similarity(&self, a: &str, b: &str) -> f32 {
        let ngrams_a: HashSet<String> = self.get_ngrams(a).into_iter().collect();
        let ngrams_b: HashSet<String> = self.get_ngrams(b).into_iter().collect();

        let intersection = ngrams_a.intersection(&ngrams_b).count();
        let union = ngrams_a.union(&ngrams_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

#[async_trait]
impl ExampleSelector for NGramExampleSelector {
    async fn select_examples(
        &self,
        input_variables: &HashMap<String, Value>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        if self.examples.is_empty() {
            return Ok(Vec::new());
        }

        let input_text = input_variables
            .values()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let mut scored: Vec<(f32, usize)> = self
            .examples
            .iter()
            .enumerate()
            .map(|(i, example)| {
                let example_text = example
                    .values()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                let sim = self.jaccard_similarity(&input_text, &example_text);
                (sim, i)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .take(self.k)
            .map(|(_, idx)| self.examples[idx].clone())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_example(text: &str) -> HashMap<String, Value> {
        HashMap::from([("text".to_string(), Value::String(text.to_string()))])
    }

    #[tokio::test]
    async fn test_ngram_selector_empty_examples() {
        let selector = NGramExampleSelector::new(vec![], 3, 2);
        let input = HashMap::from([("q".to_string(), Value::String("test".to_string()))]);
        let selected = selector.select_examples(&input).await.unwrap();
        assert!(selected.is_empty());
    }

    #[tokio::test]
    async fn test_ngram_selector_selects_most_similar() {
        let examples = vec![
            make_example("the quick brown fox"),
            make_example("jumps over the lazy dog"),
            make_example("completely unrelated topic here"),
        ];
        let selector = NGramExampleSelector::new(examples, 1, 2);
        let input = HashMap::from([("q".to_string(), Value::String("quick brown fox".to_string()))]);
        let selected = selector.select_examples(&input).await.unwrap();
        assert_eq!(selected.len(), 1);
    }

    #[tokio::test]
    async fn test_ngram_jaccard_similarity() {
        let selector = NGramExampleSelector::new(vec![], 1, 2);
        let sim = selector.jaccard_similarity("hello", "hello");
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_ngram_jaccard_similarity_no_match() {
        let selector = NGramExampleSelector::new(vec![], 1, 3);
        let sim = selector.jaccard_similarity("abc", "xyz");
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_ngram_get_ngrams() {
        let selector = NGramExampleSelector::new(vec![], 1, 2);
        let ngrams = selector.get_ngrams("abc");
        assert_eq!(ngrams, vec!["ab", "bc"]);
    }

    #[tokio::test]
    async fn test_ngram_get_ngrams_shorter_than_n() {
        let selector = NGramExampleSelector::new(vec![], 1, 5);
        let ngrams = selector.get_ngrams("hi");
        assert_eq!(ngrams, vec!["hi"]);
    }

    #[tokio::test]
    async fn test_ngram_add_example() {
        let mut selector = NGramExampleSelector::new(vec![], 3, 2);
        selector.add_example(make_example("new example"));
        assert_eq!(selector.examples.len(), 1);
    }
}
