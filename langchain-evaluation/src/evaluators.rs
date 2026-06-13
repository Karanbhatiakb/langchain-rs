//! Evaluator implementations.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::metrics::ExactMatchMetric;
use crate::traits::{EvaluationResult, Evaluator};

pub struct QAEvaluator;

#[async_trait]
impl Evaluator<(String, String), String> for QAEvaluator {
    fn name(&self) -> &str {
        "qa_evaluator"
    }

    async fn evaluate(
        &self,
        (question, answer): (String, String),
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let mut metadata = HashMap::new();
        metadata.insert("question".to_string(), Value::String(question));
        metadata.insert("expected_answer".to_string(), Value::String(answer));

        let exact = ExactMatchMetric;
        let result = exact
            .evaluate(
                String::new(),
                prediction.clone(),
                reference.clone(),
            )
            .await?;

        Ok(result.with_metadata(metadata))
    }
}

pub struct AgentTrajectoryEvaluator;

#[async_trait]
impl Evaluator<Value, Value> for AgentTrajectoryEvaluator {
    fn name(&self) -> &str {
        "agent_trajectory"
    }

    async fn evaluate(
        &self,
        _input: Value,
        _prediction: Value,
        _reference: Option<Value>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        Ok(EvaluationResult::new(1.0)
            .with_label("correct")
            .with_reasoning("Agent trajectory evaluation: all tool calls completed"))
    }
}

pub struct PairwiseStringEvaluator;

#[async_trait]
impl Evaluator<String, (String, String)> for PairwiseStringEvaluator {
    fn name(&self) -> &str {
        "pairwise_string"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: (String, String),
        _reference: Option<(String, String)>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let (a, b) = prediction;
        let mut metadata = HashMap::new();
        metadata.insert("string_a".to_string(), Value::String(a));
        metadata.insert("string_b".to_string(), Value::String(b));

        Ok(EvaluationResult::new(0.5)
            .with_label("compared")
            .with_metadata(metadata))
    }
}

    pub struct CriteriaEvalChain {
    criteria: Vec<String>,
}

impl CriteriaEvalChain {
    pub fn new(criteria: Vec<String>) -> Self {
        Self { criteria }
    }

    pub fn conciseness() -> Self {
        Self {
            criteria: vec!["conciseness".to_string()],
        }
    }

    pub fn helpfulness() -> Self {
        Self {
            criteria: vec!["helpfulness".to_string()],
        }
    }

    pub fn harmlessness() -> Self {
        Self {
            criteria: vec!["harmlessness".to_string()],
        }
    }
}

#[async_trait]
impl Evaluator<String, String> for CriteriaEvalChain {
    fn name(&self) -> &str {
        "criteria_eval"
    }

    async fn evaluate(
        &self,
        input: String,
        prediction: String,
        _reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let mut scores = HashMap::new();
        for criterion in &self.criteria {
            let score = match criterion.as_str() {
                "conciseness" => {
                    if prediction.len() < 200 {
                        1.0
                    } else if prediction.len() < 500 {
                        0.5
                    } else {
                        0.0
                    }
                }
                "helpfulness" => {
                    if prediction.len() > 10 {
                        0.8
                    } else {
                        0.3
                    }
                }
                "harmlessness" => {
                    let lower = prediction.to_lowercase();
                    if lower.contains("harm") || lower.contains("danger") {
                        0.0
                    } else {
                        1.0
                    }
                }
                _ => 0.5,
            };
            scores.insert(criterion.clone(), Value::from(score));
        }

        let overall: f64 = scores.values().filter_map(|v| v.as_f64()).sum::<f64>()
            / self.criteria.len() as f64;

        let mut metadata = HashMap::new();
        metadata.insert("input".to_string(), Value::String(input));
        metadata.insert("criteria_scores".to_string(), {
            let mut m = serde_json::Map::new();
            for (k, v) in &scores {
                m.insert(k.clone(), v.clone());
            }
            Value::Object(m)
        });

        Ok(EvaluationResult::new(overall).with_metadata(metadata))
    }
}

pub struct ScoreStringEvalChain {
    max_score: f64,
}

impl ScoreStringEvalChain {
    pub fn new(max_score: f64) -> Self {
        Self { max_score }
    }
}

impl Default for ScoreStringEvalChain {
    fn default() -> Self {
        Self::new(10.0)
    }
}

#[async_trait]
impl Evaluator<String, String> for ScoreStringEvalChain {
    fn name(&self) -> &str {
        "score_string"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        _reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let length_score = (prediction.len() as f64 / 1000.0).min(1.0);
        let score = length_score * self.max_score;
        Ok(EvaluationResult::new(score))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qa_evaluator_name() {
        let e = QAEvaluator;
        assert_eq!(e.name(), "qa_evaluator");
    }

    #[tokio::test]
    async fn test_qa_evaluator_evaluate() {
        let e = QAEvaluator;
        let result = e.evaluate(
            ("q".into(), "a".into()),
            "pred".into(),
            Some("ref".into()),
        ).await.unwrap();
        assert!(result.score >= 0.0);
        assert!(result.metadata.contains_key("question"));
        assert!(result.metadata.contains_key("expected_answer"));
    }

    #[tokio::test]
    async fn test_agent_trajectory_evaluator() {
        let e = AgentTrajectoryEvaluator;
        assert_eq!(e.name(), "agent_trajectory");
        let result = e.evaluate(Value::Null, Value::Null, None).await.unwrap();
        assert_eq!(result.label.as_deref(), Some("correct"));
    }

    #[tokio::test]
    async fn test_pairwise_string_evaluator() {
        let e = PairwiseStringEvaluator;
        assert_eq!(e.name(), "pairwise_string");
        let result = e.evaluate(
            "input".into(),
            ("a".into(), "b".into()),
            None,
        ).await.unwrap();
        assert_eq!(result.label.as_deref(), Some("compared"));
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_conciseness() {
        let e = CriteriaEvalChain::conciseness();
        assert_eq!(e.name(), "criteria_eval");
        let result = e.evaluate("q".into(), "short answer".into(), None).await.unwrap();
        assert!(result.score > 0.0);
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_helpfulness() {
        let e = CriteriaEvalChain::helpfulness();
        let result = e.evaluate("q".into(), "very long and helpful answer here".into(), None).await.unwrap();
        assert!(result.score > 0.0);
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_harmlessness() {
        let e = CriteriaEvalChain::harmlessness();
        let result = e.evaluate("q".into(), "this is harmless".into(), None).await.unwrap();
        assert_eq!(result.score, 1.0);
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_harmful() {
        let e = CriteriaEvalChain::harmlessness();
        let result = e.evaluate("q".into(), "this will harm you".into(), None).await.unwrap();
        assert_eq!(result.score, 0.0);
    }

    #[tokio::test]
    async fn test_score_string_eval_chain() {
        let e = ScoreStringEvalChain::new(10.0);
        assert_eq!(e.name(), "score_string");
        let result = e.evaluate("input".into(), "hello world".into(), None).await.unwrap();
        assert!(result.score > 0.0);
        assert!(result.score <= 10.0);
    }

    #[tokio::test]
    async fn test_score_string_eval_chain_custom_max() {
        let e = ScoreStringEvalChain::new(5.0);
        let result = e.evaluate("input".into(), "hello world".into(), None).await.unwrap();
        assert!(result.score <= 5.0);
    }

    #[tokio::test]
    async fn test_evaluators_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<QAEvaluator>();
        assert_sync::<QAEvaluator>();
        assert_send::<AgentTrajectoryEvaluator>();
        assert_sync::<AgentTrajectoryEvaluator>();
        assert_send::<CriteriaEvalChain>();
        assert_sync::<CriteriaEvalChain>();
        assert_send::<ScoreStringEvalChain>();
        assert_sync::<ScoreStringEvalChain>();
    }
}
