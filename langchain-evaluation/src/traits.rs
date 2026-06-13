//! Core [`Evaluator`] trait and [`EvaluationResult`] type.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// The result of evaluating a single prediction against a reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// A numeric score (e.g., 0.0–1.0).
    pub score: f64,
    /// An optional categorical label (e.g., "correct", "incorrect").
    pub label: Option<String>,
    /// Optional reasoning or justification for the score.
    pub reasoning: Option<String>,
    /// Additional metadata associated with the evaluation.
    pub metadata: HashMap<String, Value>,
}

impl Default for EvaluationResult {
    fn default() -> Self {
        Self {
            score: 0.0,
            label: None,
            reasoning: None,
            metadata: HashMap::new(),
        }
    }
}

impl EvaluationResult {
    /// Creates a new `EvaluationResult` with the given score.
    pub fn new(score: f64) -> Self {
        Self {
            score,
            label: None,
            reasoning: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the label (builder pattern).
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the reasoning (builder pattern).
    pub fn with_reasoning(mut self, reasoning: impl Into<String>) -> Self {
        self.reasoning = Some(reasoning.into());
        self
    }

    /// Sets metadata (builder pattern).
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Returns `true` if the result is considered passing.
    ///
    /// A result passes if the label is `"correct"` or `"pass"`, or if the
    /// score is >= 0.5.
    pub fn passed(&self) -> bool {
        self.label.as_deref() == Some("correct")
            || self.label.as_deref() == Some("pass")
            || self.score >= 0.5
    }
}

/// Trait for evaluators that judge a prediction against an optional reference.
///
/// # Type parameters
/// * `I` — Input type.
/// * `O` — Output/prediction type.
#[async_trait]
pub trait Evaluator<I, O>: Send + Sync {
    /// Returns the evaluator's name.
    fn name(&self) -> &str;
    /// Evaluates a prediction against an optional reference.
    async fn evaluate(
        &self,
        input: I,
        prediction: O,
        reference: Option<O>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluation_result_default() {
        let r = EvaluationResult::default();
        assert!((r.score - 0.0).abs() < 1e-6);
        assert!(r.label.is_none());
        assert!(r.reasoning.is_none());
        assert!(r.metadata.is_empty());
    }

    #[test]
    fn test_evaluation_result_new() {
        let r = EvaluationResult::new(0.85);
        assert!((r.score - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_evaluation_result_with_label() {
        let r = EvaluationResult::new(0.5).with_label("correct");
        assert_eq!(r.label.as_deref(), Some("correct"));
    }

    #[test]
    fn test_evaluation_result_with_reasoning() {
        let r = EvaluationResult::new(0.0).with_reasoning("bad");
        assert_eq!(r.reasoning.as_deref(), Some("bad"));
    }

    #[test]
    fn test_evaluation_result_with_metadata() {
        let mut meta = HashMap::new();
        meta.insert("k".into(), Value::String("v".into()));
        let r = EvaluationResult::new(1.0).with_metadata(meta);
        assert_eq!(r.metadata.get("k").and_then(|v| v.as_str()), Some("v"));
    }

    #[test]
    fn test_evaluation_result_passed_by_label_correct() {
        let r = EvaluationResult::new(0.0).with_label("correct");
        assert!(r.passed());
    }

    #[test]
    fn test_evaluation_result_passed_by_label_pass() {
        let r = EvaluationResult::new(0.0).with_label("pass");
        assert!(r.passed());
    }

    #[test]
    fn test_evaluation_result_passed_by_score() {
        let r = EvaluationResult::new(0.7);
        assert!(r.passed());
    }

    #[test]
    fn test_evaluation_result_not_passed() {
        let r = EvaluationResult::new(0.3).with_label("wrong");
        assert!(!r.passed());
    }

    #[test]
    fn test_evaluation_result_serialization() {
        let r = EvaluationResult::new(0.9).with_label("pass");
        let json = serde_json::to_string(&r).unwrap();
        let deserialized: EvaluationResult = serde_json::from_str(&json).unwrap();
        assert!((deserialized.score - 0.9).abs() < 1e-6);
        assert_eq!(deserialized.label.as_deref(), Some("pass"));
    }

    #[test]
    fn test_evaluator_trait_object() {
        struct TestEval;
        #[async_trait]
        impl Evaluator<String, String> for TestEval {
            fn name(&self) -> &str { "test" }
            async fn evaluate(&self, _: String, _: String, _: Option<String>) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
                Ok(EvaluationResult::new(1.0))
            }
        }
        let e: &dyn Evaluator<String, String> = &TestEval;
        assert_eq!(e.name(), "test");
    }

    #[test]
    fn test_evaluation_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<EvaluationResult>();
        assert_sync::<EvaluationResult>();
    }
}
