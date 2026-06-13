//! Maximal Marginal Relevance (MMR) example selector for diversifying results.

use crate::errors::*;
use crate::example_selectors::ExampleSelector;
use crate::utils::cosine_similarity;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct MaximalMarginalRelevanceExampleSelector {
    pub examples: Vec<HashMap<String, Value>>,
    pub k: usize,
    pub lambda: f32,
    pub example_vectors: Vec<Vec<f32>>,
}

impl MaximalMarginalRelevanceExampleSelector {
    pub fn new(
        examples: Vec<HashMap<String, Value>>,
        k: usize,
        lambda: f32,
        example_vectors: Vec<Vec<f32>>,
    ) -> Self {
        Self {
            examples,
            k,
            lambda,
            example_vectors,
        }
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn with_lambda(mut self, lambda: f32) -> Self {
        self.lambda = lambda;
        self
    }

    pub fn add_example(&mut self, example: HashMap<String, Value>, vector: Vec<f32>) {
        self.examples.push(example);
        self.example_vectors.push(vector);
    }

    fn simple_embedding(text: &str) -> Vec<f32> {
        let len = text.len() as f32;
        vec![len.cos(), len.sin(), len.fract(), len.sqrt().fract(), (len * 0.1).cos()]
    }

    fn mmr_select(
        &self,
        query_vector: &[f32],
        k: usize,
    ) -> Vec<usize> {
        if self.example_vectors.is_empty() || k == 0 {
            return Vec::new();
        }

        let n = self.example_vectors.len();
        let mut selected_indices: Vec<usize> = Vec::new();
        let mut selected_set: HashSet<usize> = HashSet::new();
        let relevance: Vec<f32> = self
            .example_vectors
            .iter()
            .map(|v| cosine_similarity(query_vector, v))
            .collect();

        while selected_indices.len() < k && selected_indices.len() < n {
            let mut best_idx = None;
            let mut best_score = f32::NEG_INFINITY;

            for i in 0..n {
                if selected_set.contains(&i) {
                    continue;
                }

                let relevance_score = relevance[i];

                let max_similarity = if selected_indices.is_empty() {
                    0.0
                } else {
                    selected_indices
                        .iter()
                        .map(|&j| cosine_similarity(&self.example_vectors[i], &self.example_vectors[j]))
                        .fold(f32::NEG_INFINITY, f32::max)
                };

                let mmr_score = self.lambda * relevance_score
                    - (1.0 - self.lambda) * max_similarity;

                if mmr_score > best_score {
                    best_score = mmr_score;
                    best_idx = Some(i);
                }
            }

            if let Some(idx) = best_idx {
                selected_indices.push(idx);
                selected_set.insert(idx);
            } else {
                break;
            }
        }

        selected_indices
    }
}

#[async_trait]
impl ExampleSelector for MaximalMarginalRelevanceExampleSelector {
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

        let query_vector = Self::simple_embedding(&input_text);
        let indices = self.mmr_select(&query_vector, self.k);

        Ok(indices
            .into_iter()
            .map(|i| self.examples[i].clone())
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
    async fn test_mmr_selector_empty_examples() {
        let selector = MaximalMarginalRelevanceExampleSelector::new(vec![], 3, 0.5, vec![]);
        let input = HashMap::from([("q".to_string(), Value::String("test".to_string()))]);
        let selected = selector.select_examples(&input).await.unwrap();
        assert!(selected.is_empty());
    }

    #[tokio::test]
    async fn test_mmr_selector_returns_k_examples() {
        let examples = vec![
            make_example("doc1"),
            make_example("doc2"),
            make_example("doc3"),
            make_example("doc4"),
            make_example("doc5"),
        ];
        let vectors: Vec<Vec<f32>> = (0..5).map(|i| vec![i as f32, 0.0, 0.0]).collect();
        let selector = MaximalMarginalRelevanceExampleSelector::new(examples, 3, 0.5, vectors);
        let input = HashMap::from([("q".to_string(), Value::String("query".to_string()))]);
        let selected = selector.select_examples(&input).await.unwrap();
        assert_eq!(selected.len(), 3);
    }

    #[tokio::test]
    async fn test_mmr_add_example() {
        let mut selector = MaximalMarginalRelevanceExampleSelector::new(vec![], 2, 0.5, vec![]);
        selector.add_example(make_example("new"), vec![1.0, 0.0]);
        assert_eq!(selector.examples.len(), 1);
        assert_eq!(selector.example_vectors.len(), 1);
    }

    #[tokio::test]
    async fn test_mmr_simple_embedding() {
        let emb = MaximalMarginalRelevanceExampleSelector::simple_embedding("hello");
        assert_eq!(emb.len(), 5);
    }

    #[tokio::test]
    async fn test_mmr_select_with_zero_k() {
        let selector = MaximalMarginalRelevanceExampleSelector::new(vec![], 0, 0.5, vec![]);
        let result = selector.mmr_select(&[1.0, 0.0], 0);
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_mmr_select_with_lambda_1() {
        let examples = vec![make_example("a"), make_example("b")];
        let vectors = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let selector = MaximalMarginalRelevanceExampleSelector::new(examples, 1, 1.0, vectors);
        let result = selector.mmr_select(&[1.0, 0.1], 1);
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_mmr_selector_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<MaximalMarginalRelevanceExampleSelector>();
        assert_sync::<MaximalMarginalRelevanceExampleSelector>();
    }
}
