//! Evaluation runner — orchestrates running evaluations over datasets with concurrency.

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::datasets::Dataset;
use crate::traits::Evaluator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerEvalResult {
    pub input: Value,
    pub prediction: Value,
    pub reference: Option<Value>,
    pub score: f64,
    pub label: Option<String>,
    pub reasoning: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerEvalReport {
    pub name: String,
    pub evaluator_name: String,
    pub dataset_name: String,
    pub total_examples: usize,
    pub completed_examples: usize,
    pub failed_examples: usize,
    pub mean_score: f64,
    pub median_score: f64,
    pub min_score: f64,
    pub max_score: f64,
    pub std_dev: f64,
    pub p95_score: f64,
    pub pass_count: usize,
    pub fail_count: usize,
    pub pass_rate: f64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: u64,
    pub results: Vec<RunnerEvalResult>,
}

impl RunnerEvalReport {
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path.as_ref(), json)?;
        Ok(())
    }

    pub fn save_csv(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(path.as_ref())?;
        wtr.write_record(&[
            "input", "prediction", "reference", "score", "label", "reasoning", "duration_ms", "error",
        ])?;
        for result in &self.results {
            wtr.write_record(&[
                result.input.to_string(),
                result.prediction.to_string(),
                result.reference.as_ref().map(|v| v.to_string()).unwrap_or_default(),
                result.score.to_string(),
                result.label.clone().unwrap_or_default(),
                result.reasoning.clone().unwrap_or_default(),
                result.duration_ms.to_string(),
                result.error.clone().unwrap_or_default(),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }
}

pub struct EvaluationRunner<I, O> {
    evaluator: Arc<dyn Evaluator<I, O>>,
    max_concurrency: usize,
    input_fn: Box<dyn Fn(&Value) -> I + Send + Sync>,
    prediction_fn: Box<dyn Fn(&Value) -> O + Send + Sync>,
    reference_fn: Box<dyn Fn(&Value) -> Option<O> + Send + Sync>,
}

impl<I, O> EvaluationRunner<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    pub fn new(
        evaluator: Arc<dyn Evaluator<I, O>>,
    ) -> Self
    where
        I: From<Value>,
        O: From<Value>,
    {
        Self {
            evaluator,
            max_concurrency: 8,
            input_fn: Box::new(|v: &Value| I::from(v.clone())),
            prediction_fn: Box::new(|v: &Value| O::from(v.clone())),
            reference_fn: Box::new(|_v: &Value| None),
        }
    }

    pub fn with_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max.max(1);
        self
    }

    pub fn with_input_fn(mut self, f: impl Fn(&Value) -> I + Send + Sync + 'static) -> Self {
        self.input_fn = Box::new(f);
        self
    }

    pub fn with_prediction_fn(mut self, f: impl Fn(&Value) -> O + Send + Sync + 'static) -> Self {
        self.prediction_fn = Box::new(f);
        self
    }

    pub fn with_reference_fn(mut self, f: impl Fn(&Value) -> Option<O> + Send + Sync + 'static) -> Self {
        self.reference_fn = Box::new(f);
        self
    }

    pub async fn run(&self, dataset: &Dataset) -> RunnerEvalReport {
        let start = std::time::Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));

        let mut handles = Vec::new();

        for example in &dataset.examples {
            let evaluator = self.evaluator.clone();
            let semaphore = semaphore.clone();
            let input_fn = &self.input_fn;
            let prediction_fn = &self.prediction_fn;
            let reference_fn = &self.reference_fn;

            let input_val = example.input.clone();
            let output_val = example.output.clone();
            let input = input_fn(&input_val);
            let prediction = prediction_fn(&output_val);
            let reference = reference_fn(&output_val);

            handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let eval_start = std::time::Instant::now();
                let result = evaluator.evaluate(input, prediction, reference).await;
                let duration_ms = eval_start.elapsed().as_millis() as u64;
                (input_val, output_val, result, duration_ms)
            }));
        }

        let raw_results = join_all(handles).await;
        let mut results = Vec::new();
        let mut completed = 0usize;
        let mut failed = 0usize;

        for res in raw_results {
            match res {
                Ok((input, output, eval_result, duration_ms)) => {
                    match eval_result {
                        Ok(er) => {
                            results.push(RunnerEvalResult {
                                input,
                                prediction: output,
                                reference: None,
                                score: er.score,
                                label: er.label,
                                reasoning: er.reasoning,
                                metadata: er.metadata,
                                duration_ms,
                                error: None,
                            });
                            completed += 1;
                        }
                        Err(e) => {
                            results.push(RunnerEvalResult {
                                input,
                                prediction: output,
                                reference: None,
                                score: 0.0,
                                label: Some("error".to_string()),
                                reasoning: None,
                                metadata: HashMap::new(),
                                duration_ms,
                                error: Some(e.to_string()),
                            });
                            failed += 1;
                            completed += 1;
                        }
                    }
                }
                Err(e) => {
                    results.push(RunnerEvalResult {
                        input: Value::Null,
                        prediction: Value::Null,
                        reference: None,
                        score: 0.0,
                        label: Some("panic".to_string()),
                        reasoning: None,
                        metadata: HashMap::new(),
                        duration_ms: 0,
                        error: Some(e.to_string()),
                    });
                    failed += 1;
                }
            }
        }

        let total_duration_ms = start.elapsed().as_millis() as u64;
        let total = results.len();
        let scores: Vec<f64> = results.iter().map(|r| r.score).collect();

        let mean_score = if total > 0 { scores.iter().sum::<f64>() / total as f64 } else { 0.0 };
        let mut sorted_scores = scores.clone();
        sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_score = if sorted_scores.is_empty() { 0.0 } else { sorted_scores[sorted_scores.len() / 2] };
        let min_score = sorted_scores.first().copied().unwrap_or(0.0);
        let max_score = sorted_scores.last().copied().unwrap_or(0.0);
        let p95_score = if sorted_scores.is_empty() { 0.0 } else {
            let idx = ((total as f64) * 0.95) as usize;
            sorted_scores[idx.min(sorted_scores.len() - 1)]
        };

        let variance = if total > 0 {
            scores.iter().map(|s| (s - mean_score).powi(2)).sum::<f64>() / total as f64
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        let pass_count = results.iter().filter(|r| r.score >= 0.5).count();
        let fail_count = total - pass_count;
        let pass_rate = if total > 0 { pass_count as f64 / total as f64 } else { 0.0 };
        let avg_duration_ms = if total > 0 { total_duration_ms / total as u64 } else { 0 };

        RunnerEvalReport {
            name: format!("{}_eval", dataset.name),
            evaluator_name: self.evaluator.name().to_string(),
            dataset_name: dataset.name.clone(),
            total_examples: total,
            completed_examples: completed,
            failed_examples: failed,
            mean_score,
            median_score,
            min_score,
            max_score,
            std_dev,
            p95_score,
            pass_count,
            fail_count,
            pass_rate,
            total_duration_ms,
            avg_duration_ms,
            results,
        }
    }
}
