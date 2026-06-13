use std::collections::HashMap;

use langchain_evaluation::datasets::{Dataset, DatasetExample};
use langchain_evaluation::metrics::*;
use langchain_evaluation::traits::{EvaluationResult, Evaluator};
use langchain_evaluation::evaluators::*;
use langchain_evaluation::runners::run_evaluation;
use serde_json::Value;

#[test]
fn test_evaluation_result_new() {
    let result = EvaluationResult::new(0.85);
    assert!((result.score - 0.85).abs() < f64::EPSILON);
    assert!(result.label.is_none());
    assert!(result.reasoning.is_none());
}

#[test]
fn test_evaluation_result_with_label() {
    let result = EvaluationResult::new(1.0).with_label("correct");
    assert_eq!(result.label.as_deref(), Some("correct"));
}

#[test]
fn test_evaluation_result_with_reasoning() {
    let result = EvaluationResult::new(0.5).with_reasoning("partial match");
    assert_eq!(result.reasoning.as_deref(), Some("partial match"));
}

#[test]
fn test_evaluation_result_with_metadata() {
    let meta = HashMap::from([("key".to_string(), Value::String("val".to_string()))]);
    let result = EvaluationResult::new(1.0).with_metadata(meta);
    assert!(result.metadata.contains_key("key"));
}

#[test]
fn test_evaluation_result_passed() {
    let r1 = EvaluationResult::new(0.7);
    assert!(r1.passed());
    let r2 = EvaluationResult::new(0.3);
    assert!(!r2.passed());
    let r3 = EvaluationResult::new(1.0).with_label("correct");
    assert!(r3.passed());
    let r4 = EvaluationResult::new(1.0).with_label("pass");
    assert!(r4.passed());
}

#[test]
fn test_evaluation_result_default() {
    let result = EvaluationResult::default();
    assert_eq!(result.score, 0.0);
    assert!(result.label.is_none());
}

#[test]
fn test_dataset_example_new() {
    let ex = DatasetExample::new(Value::String("q".to_string()), Value::String("a".to_string()));
    assert_eq!(ex.input, Value::String("q".to_string()));
    assert_eq!(ex.output, Value::String("a".to_string()));
    assert!(ex.metadata.is_empty());
}

#[test]
fn test_dataset_example_with_metadata() {
    let meta = HashMap::from([("src".to_string(), Value::String("wiki".to_string()))]);
    let ex = DatasetExample::new(Value::String("q".to_string()), Value::String("a".to_string()))
        .with_metadata(meta);
    assert!(ex.metadata.contains_key("src"));
}

#[test]
fn test_dataset_new() {
    let ds = Dataset::new("test_ds");
    assert_eq!(ds.name, "test_ds");
    assert!(ds.description.is_none());
    assert!(ds.is_empty());
}

#[test]
fn test_dataset_with_description() {
    let ds = Dataset::new("ds").with_description("A test dataset");
    assert_eq!(ds.description.as_deref(), Some("A test dataset"));
}

#[test]
fn test_dataset_add_example() {
    let mut ds = Dataset::new("test");
    ds.add_example(DatasetExample::new(
        Value::String("q1".to_string()),
        Value::String("a1".to_string()),
    ));
    assert_eq!(ds.len(), 1);
    assert!(!ds.is_empty());
}

#[test]
fn test_dataset_from_list() {
    let examples = vec![
        DatasetExample::new(Value::String("q1".to_string()), Value::String("a1".to_string())),
        DatasetExample::new(Value::String("q2".to_string()), Value::String("a2".to_string())),
    ];
    let ds = Dataset::from_list(examples);
    assert_eq!(ds.len(), 2);
}

#[test]
fn test_dataset_iter() {
    let mut ds = Dataset::new("test");
    ds.add_example(DatasetExample::new(Value::String("q".to_string()), Value::String("a".to_string())));
    let count = ds.iter().count();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_exact_match_metric() {
    let metric = ExactMatchMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "hello".to_string(),
        Some("hello".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
    assert_eq!(result.label.as_deref(), Some("correct"));
}

#[tokio::test]
async fn test_exact_match_metric_fail() {
    let metric = ExactMatchMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "hello".to_string(),
        Some("world".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
}

#[tokio::test]
async fn test_exact_match_trims() {
    let metric = ExactMatchMetric;
    let result = metric.evaluate(
        "q".to_string(),
        " hello ".to_string(),
        Some("hello".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
}

#[tokio::test]
async fn test_exact_match_no_reference() {
    let metric = ExactMatchMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "hello".to_string(),
        None,
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
}

#[tokio::test]
async fn test_contains_metric() {
    let metric = ContainsMetric::new();
    let result = metric.evaluate(
        "q".to_string(),
        "hello world".to_string(),
        Some("world".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
}

#[tokio::test]
async fn test_contains_metric_case_insensitive() {
    let metric = ContainsMetric::new();
    let result = metric.evaluate(
        "q".to_string(),
        "Hello World".to_string(),
        Some("world".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
}

#[tokio::test]
async fn test_contains_metric_case_sensitive() {
    let metric = ContainsMetric::new().case_sensitive(true);
    let result = metric.evaluate(
        "q".to_string(),
        "Hello World".to_string(),
        Some("world".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
}

#[tokio::test]
async fn test_regex_metric() {
    let metric = RegexMetric::new(r"\d+").unwrap();
    let result = metric.evaluate(
        "q".to_string(),
        "There are 42 items".to_string(),
        None,
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
}

#[tokio::test]
async fn test_regex_metric_no_match() {
    let metric = RegexMetric::new(r"\d+").unwrap();
    let result = metric.evaluate(
        "q".to_string(),
        "no numbers here".to_string(),
        None,
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
}

#[tokio::test]
async fn test_f1_score_metric() {
    let metric = F1ScoreMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "the cat sat on the mat".to_string(),
        Some("the cat on the mat".to_string()),
    ).await.unwrap();
    assert!(result.score > 0.5);
}

#[tokio::test]
async fn test_f1_score_identical() {
    let metric = F1ScoreMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "hello world".to_string(),
        Some("hello world".to_string()),
    ).await.unwrap();
    assert!((result.score - 1.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_bleu_score_metric() {
    let metric = BLEUScoreMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "the cat sat on the mat".to_string(),
        Some("the cat sat on the mat".to_string()),
    ).await.unwrap();
    assert!((result.score - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_rouge_score_metric() {
    let metric = ROUGEScoreMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "the cat".to_string(),
        Some("the cat".to_string()),
    ).await.unwrap();
    assert!((result.score - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_precision_metric() {
    let metric = PrecisionMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "cat dog".to_string(),
        Some("cat dog bird".to_string()),
    ).await.unwrap();
    assert!((result.score - 1.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_recall_metric() {
    let metric = RecallMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "cat dog bird".to_string(),
        Some("cat dog".to_string()),
    ).await.unwrap();
    assert!((result.score - 1.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_accuracy_metric() {
    let metric = AccuracyMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "yes".to_string(),
        Some("YES".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
}

#[tokio::test]
async fn test_string_distance_metric() {
    let metric = StringDistanceMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "hello".to_string(),
        Some("hello".to_string()),
    ).await.unwrap();
    assert!((result.score - 1.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_string_distance_different() {
    let metric = StringDistanceMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "abc".to_string(),
        Some("xyz".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
}

#[tokio::test]
async fn test_embedding_distance_metric() {
    let metric = EmbeddingDistanceMetric;
    let result = metric.evaluate(
        "q".to_string(),
        "pred".to_string(),
        Some("ref".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
    assert!(result.reasoning.is_some());
}

#[tokio::test]
async fn test_llm_as_judge_metric() {
    let metric = LLMAsJudgeMetric::new("correctness");
    let result = metric.evaluate(
        "q".to_string(),
        "pred".to_string(),
        Some("ref".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
    assert!(result.reasoning.is_some());
    assert_eq!(metric.name(), "llm_as_judge");
}

#[tokio::test]
async fn test_qa_evaluator() {
    let eval = QAEvaluator;
    let result = eval.evaluate(
        ("What is 2+2?".to_string(), "4".to_string()),
        "4".to_string(),
        Some("4".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
    assert!(result.metadata.contains_key("question"));
    assert_eq!(eval.name(), "qa_evaluator");
}

#[tokio::test]
async fn test_qa_evaluator_wrong() {
    let eval = QAEvaluator;
    let result = eval.evaluate(
        ("What is 2+2?".to_string(), "4".to_string()),
        "5".to_string(),
        Some("4".to_string()),
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
}

#[tokio::test]
async fn test_agent_trajectory_evaluator() {
    let eval = AgentTrajectoryEvaluator;
    let result = eval.evaluate(
        serde_json::json!({"input": "test"}),
        serde_json::json!({"output": "test"}),
        None,
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
    assert_eq!(eval.name(), "agent_trajectory");
}

#[tokio::test]
async fn test_pairwise_string_evaluator() {
    let eval = PairwiseStringEvaluator;
    let result = eval.evaluate(
        "q".to_string(),
        ("a".to_string(), "b".to_string()),
        None,
    ).await.unwrap();
    assert_eq!(result.score, 0.5);
    assert!(result.metadata.contains_key("string_a"));
}

#[tokio::test]
async fn test_criteria_eval_chain_conciseness() {
    let eval = CriteriaEvalChain::conciseness();
    let result = eval.evaluate(
        "q".to_string(),
        "short answer".to_string(),
        None,
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
    assert_eq!(eval.name(), "criteria_eval");
}

#[tokio::test]
async fn test_criteria_eval_chain_harmlessness() {
    let eval = CriteriaEvalChain::harmlessness();
    let result = eval.evaluate(
        "q".to_string(),
        "This is a safe response".to_string(),
        None,
    ).await.unwrap();
    assert_eq!(result.score, 1.0);
}

#[tokio::test]
async fn test_criteria_eval_chain_harmlessness_danger() {
    let eval = CriteriaEvalChain::harmlessness();
    let result = eval.evaluate(
        "q".to_string(),
        "This is harmful and dangerous".to_string(),
        None,
    ).await.unwrap();
    assert_eq!(result.score, 0.0);
}

#[tokio::test]
async fn test_criteria_eval_chain_helpfulness() {
    let eval = CriteriaEvalChain::helpfulness();
    let result = eval.evaluate(
        "q".to_string(),
        "This is a helpful detailed response".to_string(),
        None,
    ).await.unwrap();
    assert!((result.score - 0.8).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_score_string_eval_chain() {
    let eval = ScoreStringEvalChain::new(10.0);
    let result = eval.evaluate(
        "q".to_string(),
        "a".to_string(),
        None,
    ).await.unwrap();
    assert!(result.score >= 0.0);
    assert_eq!(eval.name(), "score_string");
}

#[tokio::test]
async fn test_score_string_eval_chain_default() {
    let eval = ScoreStringEvalChain::default();
    let result = eval.evaluate(
        "q".to_string(),
        "a".to_string(),
        None,
    ).await.unwrap();
    assert!(result.score <= 10.0);
}

#[tokio::test]
async fn test_run_evaluation() {
    let metric = ExactMatchMetric;
    let mut ds = Dataset::new("test");
    ds.add_example(DatasetExample::new(
        Value::String("q1".to_string()),
        Value::String("hello".to_string()),
    ));
    ds.add_example(DatasetExample::new(
        Value::String("q2".to_string()),
        Value::String("world".to_string()),
    ));
    let report = run_evaluation(
        &metric,
        &ds,
        |v: &Value| v.as_str().unwrap_or("").to_string(),
        |v: &Value| v.as_str().unwrap_or("").to_string(),
        |v: &Value| Some(v.as_str().unwrap_or("").to_string()),
    ).await;
    assert_eq!(report.total_examples, 2);
    assert_eq!(report.evaluator_name, "exact_match");
}
