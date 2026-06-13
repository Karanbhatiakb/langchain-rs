//! Evaluation metric implementations.

use async_trait::async_trait;
use regex::Regex;
use std::collections::{HashMap, HashSet};

use crate::traits::{EvaluationResult, Evaluator};

pub struct ExactMatchMetric;

#[async_trait]
impl Evaluator<String, String> for ExactMatchMetric {
    fn name(&self) -> &str {
        "exact_match"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) if prediction.trim() == r.trim() => 1.0,
            _ => 0.0,
        };
        let label = if score > 0.5 { "correct" } else { "incorrect" };
        Ok(EvaluationResult::new(score).with_label(label))
    }
}

pub struct ContainsMetric {
    pub case_sensitive: bool,
}

impl Default for ContainsMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainsMetric {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
        }
    }

    pub fn case_sensitive(mut self, val: bool) -> Self {
        self.case_sensitive = val;
        self
    }
}

#[async_trait]
impl Evaluator<String, String> for ContainsMetric {
    fn name(&self) -> &str {
        "contains"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) => {
                if self.case_sensitive {
                    if prediction.contains(r) {
                        1.0
                    } else {
                        0.0
                    }
                } else if prediction.to_lowercase().contains(&r.to_lowercase()) {
                    1.0
                } else {
                    0.0
                }
            }
            None => 0.0,
        };
        let label = if score > 0.5 { "correct" } else { "incorrect" };
        Ok(EvaluationResult::new(score).with_label(label))
    }
}

pub struct RegexMetric {
    pattern: Regex,
}

impl RegexMetric {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
        })
    }
}

#[async_trait]
impl Evaluator<String, String> for RegexMetric {
    fn name(&self) -> &str {
        "regex_match"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        _reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = if self.pattern.is_match(&prediction) {
            1.0
        } else {
            0.0
        };
        let label = if score > 0.5 { "correct" } else { "incorrect" };
        Ok(EvaluationResult::new(score).with_label(label))
    }
}

pub struct F1ScoreMetric;

#[async_trait]
impl Evaluator<String, String> for F1ScoreMetric {
    fn name(&self) -> &str {
        "f1_score"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) => compute_f1(&prediction, r),
            None => 0.0,
        };
        Ok(EvaluationResult::new(score))
    }
}

pub fn compute_f1(prediction: &str, reference: &str) -> f64 {
    let pred_tokens: HashSet<String> = prediction
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();
    let ref_tokens: HashSet<String> = reference
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    if pred_tokens.is_empty() && ref_tokens.is_empty() {
        return 1.0;
    }
    if pred_tokens.is_empty() || ref_tokens.is_empty() {
        return 0.0;
    }

    let intersection: HashSet<_> = pred_tokens.intersection(&ref_tokens).cloned().collect();
    let precision = intersection.len() as f64 / pred_tokens.len() as f64;
    let recall = intersection.len() as f64 / ref_tokens.len() as f64;

    if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    }
}

pub struct BLEUScoreMetric;

#[async_trait]
impl Evaluator<String, String> for BLEUScoreMetric {
    fn name(&self) -> &str {
        "bleu_score"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) => compute_simple_bleu(&prediction, r),
            None => 0.0,
        };
        Ok(EvaluationResult::new(score))
    }
}

fn compute_simple_bleu(prediction: &str, reference: &str) -> f64 {
    let pred_tokens: Vec<&str> = prediction.split_whitespace().collect();
    let ref_tokens: Vec<&str> = reference.split_whitespace().collect();

    if pred_tokens.is_empty() || ref_tokens.is_empty() {
        return 0.0;
    }

    let pred_counts: HashMap<&str, usize> = {
        let mut m = HashMap::new();
        for &t in &pred_tokens {
            *m.entry(t).or_insert(0) += 1;
        }
        m
    };
    let ref_counts: HashMap<&str, usize> = {
        let mut m = HashMap::new();
        for &t in &ref_tokens {
            *m.entry(t).or_insert(0) += 1;
        }
        m
    };

    let mut matches = 0usize;
    let mut total = 0usize;
    for (token, pred_count) in &pred_counts {
        let ref_count = ref_counts.get(token).copied().unwrap_or(0);
        matches += (*pred_count).min(ref_count);
        total += pred_count;
    }

    let precision = if total == 0 {
        0.0
    } else {
        matches as f64 / total as f64
    };

    let bp = if pred_tokens.len() < ref_tokens.len() {
        (1.0 - ref_tokens.len() as f64 / pred_tokens.len() as f64).exp()
    } else {
        1.0
    };

    bp * precision
}

pub struct ROUGEScoreMetric;

#[async_trait]
impl Evaluator<String, String> for ROUGEScoreMetric {
    fn name(&self) -> &str {
        "rouge_score"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) => compute_simple_rouge_l(&prediction, r),
            None => 0.0,
        };
        Ok(EvaluationResult::new(score))
    }
}

fn compute_simple_rouge_l(prediction: &str, reference: &str) -> f64 {
    let pred_tokens: Vec<char> = prediction.chars().collect();
    let ref_tokens: Vec<char> = reference.chars().collect();

    let m = pred_tokens.len();
    let n = ref_tokens.len();
    if m == 0 || n == 0 {
        return 0.0;
    }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if pred_tokens[i - 1] == ref_tokens[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let lcs = dp[m][n] as f64;
    let recall = lcs / n as f64;
    let precision = lcs / m as f64;
    if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    }
}

pub struct PrecisionMetric;

#[async_trait]
impl Evaluator<String, String> for PrecisionMetric {
    fn name(&self) -> &str {
        "precision"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) => compute_precision(&prediction, r),
            None => 0.0,
        };
        Ok(EvaluationResult::new(score))
    }
}

fn compute_precision(prediction: &str, reference: &str) -> f64 {
    let pred_tokens: HashSet<String> = prediction
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();
    let ref_tokens: HashSet<String> = reference
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    if pred_tokens.is_empty() {
        return 0.0;
    }

    let intersection = pred_tokens.intersection(&ref_tokens).count();
    intersection as f64 / pred_tokens.len() as f64
}

pub struct RecallMetric;

#[async_trait]
impl Evaluator<String, String> for RecallMetric {
    fn name(&self) -> &str {
        "recall"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) => compute_recall(&prediction, r),
            None => 0.0,
        };
        Ok(EvaluationResult::new(score))
    }
}

fn compute_recall(prediction: &str, reference: &str) -> f64 {
    let pred_tokens: HashSet<String> = prediction
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();
    let ref_tokens: HashSet<String> = reference
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    if ref_tokens.is_empty() {
        return 0.0;
    }

    let intersection = pred_tokens.intersection(&ref_tokens).count();
    intersection as f64 / ref_tokens.len() as f64
}

pub struct AccuracyMetric;

#[async_trait]
impl Evaluator<String, String> for AccuracyMetric {
    fn name(&self) -> &str {
        "accuracy"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) if prediction.trim().to_lowercase() == r.trim().to_lowercase() => 1.0,
            _ => 0.0,
        };
        Ok(EvaluationResult::new(score))
    }
}

pub struct StringDistanceMetric;

#[async_trait]
impl Evaluator<String, String> for StringDistanceMetric {
    fn name(&self) -> &str {
        "levenshtein"
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let score = match reference {
            Some(ref r) => {
                let dist = levenshtein_distance(&prediction, r);
                let max_len = prediction.len().max(r.len());
                if max_len == 0 {
                    1.0
                } else {
                    1.0 - dist as f64 / max_len as f64
                }
            }
            None => 0.0,
        };
        Ok(EvaluationResult::new(score))
    }
}

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

pub struct EmbeddingDistanceMetric;

#[async_trait]
impl Evaluator<String, String> for EmbeddingDistanceMetric {
    fn name(&self) -> &str {
        "embedding_distance"
    }

    async fn evaluate(
        &self,
        _input: String,
        _prediction: String,
        _reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        Ok(EvaluationResult::new(0.0).with_reasoning(
            "Embedding distance requires an embedding provider; use with .with_embeddings()",
        ))
    }
}

pub struct LLMAsJudgeMetric {
    criteria: String,
}

impl LLMAsJudgeMetric {
    pub fn new(criteria: impl Into<String>) -> Self {
        Self {
            criteria: criteria.into(),
        }
    }
}

#[async_trait]
impl Evaluator<String, String> for LLMAsJudgeMetric {
    fn name(&self) -> &str {
        "llm_as_judge"
    }

    async fn evaluate(
        &self,
        _input: String,
        _prediction: String,
        _reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        Ok(EvaluationResult::new(0.0).with_reasoning(format!(
            "LLM-as-judge requires a configured LLM; criteria: {}",
            self.criteria
        )))
    }
}
