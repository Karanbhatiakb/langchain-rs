//! Evaluation runner — orchestrates eval runs.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::datasets::Dataset;
use crate::traits::{EvaluationResult, Evaluator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub input: Value,
    pub prediction: Value,
    pub reference: Option<Value>,
    pub score: f64,
    pub label: Option<String>,
    pub reasoning: Option<String>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub name: String,
    pub evaluator_name: String,
    pub total_examples: usize,
    pub mean_score: f64,
    pub median_score: f64,
    pub min_score: f64,
    pub max_score: f64,
    pub std_dev: f64,
    pub pass_count: usize,
    pub fail_count: usize,
    pub pass_rate: f64,
    pub results: Vec<EvalResult>,
}

impl EvalReport {
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path.as_ref(), json)?;
        Ok(())
    }

    pub fn save_csv(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(path.as_ref())?;
        wtr.write_record(&[
            "input",
            "prediction",
            "reference",
            "score",
            "label",
            "reasoning",
        ])?;
        for result in &self.results {
            wtr.write_record(&[
                result.input.to_string(),
                result.prediction.to_string(),
                result
                    .reference
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                result.score.to_string(),
                result.label.clone().unwrap_or_default(),
                result.reasoning.clone().unwrap_or_default(),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }
}

#[allow(dead_code)]
pub struct EvalRunner<I, O> {
    evaluator: Box<dyn Evaluator<I, O>>,
    dataset: Dataset,
}

impl<I, O> EvalRunner<I, O> {
    pub fn new(evaluator: Box<dyn Evaluator<I, O>>, dataset: Dataset) -> Self {
        Self {
            evaluator,
            dataset,
        }
    }
}

pub async fn run_evaluation<I, O>(
    evaluator: &dyn Evaluator<I, O>,
    dataset: &Dataset,
    input_fn: impl Fn(&Value) -> I,
    prediction_fn: impl Fn(&Value) -> O,
    reference_fn: impl Fn(&Value) -> Option<O>,
) -> EvalReport {
    let mut results = Vec::new();
    let mut scores = Vec::new();

    for example in &dataset.examples {
        let input = input_fn(&example.input);
        let prediction = prediction_fn(&example.output);
        let reference = reference_fn(&example.output);

        let eval_result = evaluator
            .evaluate(input, prediction, reference)
            .await
            .unwrap_or_else(|e| {
                EvaluationResult::new(0.0)
                    .with_reasoning(format!("Evaluation error: {}", e))
            });

        scores.push(eval_result.score);

        results.push(EvalResult {
            input: example.input.clone(),
            prediction: example.output.clone(),
            reference: None,
            score: eval_result.score,
            label: eval_result.label.clone(),
            reasoning: eval_result.reasoning.clone(),
            metadata: eval_result.metadata.clone(),
        });
    }

    let total = results.len();
    let mean_score = if total > 0 {
        scores.iter().sum::<f64>() / total as f64
    } else {
        0.0
    };

    let mut sorted = scores.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_score = if sorted.is_empty() {
        0.0
    } else {
        sorted[sorted.len() / 2]
    };
    let min_score = sorted.first().copied().unwrap_or(0.0);
    let max_score = sorted.last().copied().unwrap_or(0.0);

    let variance = if total > 0 {
        scores
            .iter()
            .map(|s| (s - mean_score).powi(2))
            .sum::<f64>()
            / total as f64
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let pass_count = results.iter().filter(|r| r.score >= 0.5).count();
    let fail_count = total - pass_count;
    let pass_rate = if total > 0 {
        pass_count as f64 / total as f64
    } else {
        0.0
    };

    EvalReport {
        name: dataset.name.clone(),
        evaluator_name: evaluator.name().to_string(),
        total_examples: total,
        mean_score,
        median_score,
        min_score,
        max_score,
        std_dev,
        pass_count,
        fail_count,
        pass_rate,
        results,
    }
}
