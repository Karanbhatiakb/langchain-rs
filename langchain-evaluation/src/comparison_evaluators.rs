//! Pairwise comparison evaluators — compare two predictions head-to-head.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::traits::{EvaluationResult, Evaluator};

pub struct PairwiseStringEvalChain {
    criterion: String,
}

impl PairwiseStringEvalChain {
    pub fn new(criterion: impl Into<String>) -> Self {
        Self {
            criterion: criterion.into(),
        }
    }

    pub fn correctness() -> Self {
        Self::new("correctness")
    }

    pub fn helpfulness() -> Self {
        Self::new("helpfulness")
    }

    pub fn detail() -> Self {
        Self::new("detail")
    }
}

impl Default for PairwiseStringEvalChain {
    fn default() -> Self {
        Self::new("overall")
    }
}

pub struct PairwiseComparison {
    pub prediction_a: String,
    pub prediction_b: String,
}

#[async_trait]
impl Evaluator<String, PairwiseComparison> for PairwiseStringEvalChain {
    fn name(&self) -> &str {
        "pairwise_string_eval"
    }

    async fn evaluate(
        &self,
        input: String,
        prediction: PairwiseComparison,
        reference: Option<PairwiseComparison>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let a = &prediction.prediction_a;
        let b = &prediction.prediction_b;

        let (score, label, reasoning) = match self.criterion.as_str() {
            "correctness" => {
                if let Some(ref_pair) = reference.as_ref() {
                    let dist_a = levenshtein_distance(a, &ref_pair.prediction_a);
                    let dist_b = levenshtein_distance(b, &ref_pair.prediction_b);
                    if dist_a <= dist_b {
                        (0.7, "a_preferred", format!("A is closer to reference (dist_a={}, dist_b={})", dist_a, dist_b))
                    } else {
                        (0.3, "b_preferred", format!("B is closer to reference (dist_a={}, dist_b={})", dist_a, dist_b))
                    }
                } else {
                    let score_a = compute_text_quality(a);
                    let score_b = compute_text_quality(b);
                    if score_a >= score_b {
                        (0.7, "a_preferred", format!("A has higher quality score ({:.2} vs {:.2})", score_a, score_b))
                    } else {
                        (0.3, "b_preferred", format!("B has higher quality score ({:.2} vs {:.2})", score_b, score_a))
                    }
                }
            }
            "helpfulness" => {
                let len_a = a.split_whitespace().count() as f64;
                let len_b = b.split_whitespace().count() as f64;
                let quality_a = compute_text_quality(a);
                let quality_b = compute_text_quality(b);
                let score_a = quality_a * (1.0 + (len_a / 100.0).min(1.0));
                let score_b = quality_b * (1.0 + (len_b / 100.0).min(1.0));
                if score_a >= score_b {
                    (0.7, "a_preferred", format!("A is more helpful ({:.2} vs {:.2})", score_a, score_b))
                } else {
                    (0.3, "b_preferred", format!("B is more helpful ({:.2} vs {:.2})", score_b, score_a))
                }
            }
            "detail" => {
                let len_a = a.len();
                let len_b = b.len();
                if len_a >= len_b {
                    (0.7, "a_preferred", format!("A is more detailed ({} vs {} chars)", len_a, len_b))
                } else {
                    (0.3, "b_preferred", format!("B is more detailed ({} vs {} chars)", len_b, len_a))
                }
            }
            _ => {
                let score_a = compute_text_quality(a);
                let score_b = compute_text_quality(b);
                if score_a >= score_b {
                    (0.7, "a_preferred", format!("A preferred overall ({:.2} vs {:.2})", score_a, score_b))
                } else {
                    (0.3, "b_preferred", format!("B preferred overall ({:.2} vs {:.2})", score_b, score_a))
                }
            }
        };

        let mut metadata = HashMap::new();
        metadata.insert("input".to_string(), Value::String(input));
        metadata.insert("prediction_a".to_string(), Value::String(prediction.prediction_a.clone()));
        metadata.insert("prediction_b".to_string(), Value::String(prediction.prediction_b.clone()));
        metadata.insert("criterion".to_string(), Value::String(self.criterion.clone()));

        Ok(EvaluationResult::new(score)
            .with_label(label)
            .with_reasoning(reasoning)
            .with_metadata(metadata))
    }
}

fn compute_text_quality(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    let words = text.split_whitespace().count() as f64;
    let avg_word_len = if words > 0.0 {
        text.len() as f64 / words
    } else {
        0.0
    };
    let length_score = (words / 50.0).min(1.0);
    let avg_word_score = if avg_word_len > 2.0 && avg_word_len < 10.0 { 1.0 } else { 0.5 };
    0.6 * length_score + 0.4 * avg_word_score
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ChatPrediction {
    pub messages: Vec<ChatMessage>,
}

pub struct PairwiseChatEvalChain {
    criterion: String,
}

impl PairwiseChatEvalChain {
    pub fn new(criterion: impl Into<String>) -> Self {
        Self {
            criterion: criterion.into(),
        }
    }

    pub fn coherence() -> Self {
        Self::new("coherence")
    }

    pub fn relevance() -> Self {
        Self::new("relevance")
    }
}

impl Default for PairwiseChatEvalChain {
    fn default() -> Self {
        Self::new("overall")
    }
}

#[async_trait]
impl Evaluator<String, (ChatPrediction, ChatPrediction)> for PairwiseChatEvalChain {
    fn name(&self) -> &str {
        "pairwise_chat_eval"
    }

    async fn evaluate(
        &self,
        input: String,
        prediction: (ChatPrediction, ChatPrediction),
        _reference: Option<(ChatPrediction, ChatPrediction)>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let (pred_a, pred_b) = prediction;

        let content_a: String = pred_a.messages.iter().map(|m| m.content.as_str()).collect::<Vec<_>>().join(" ");
        let content_b: String = pred_b.messages.iter().map(|m| m.content.as_str()).collect::<Vec<_>>().join(" ");

        let (score, label, reasoning) = match self.criterion.as_str() {
            "coherence" => {
                let score_a = compute_coherence(&content_a);
                let score_b = compute_coherence(&content_b);
                if score_a >= score_b {
                    (0.7, "a_preferred", format!("A is more coherent ({:.2} vs {:.2})", score_a, score_b))
                } else {
                    (0.3, "b_preferred", format!("B is more coherent ({:.2} vs {:.2})", score_b, score_a))
                }
            }
            "relevance" => {
                let score_a = compute_relevance(&input, &content_a);
                let score_b = compute_relevance(&input, &content_b);
                if score_a >= score_b {
                    (0.7, "a_preferred", format!("A is more relevant ({:.2} vs {:.2})", score_a, score_b))
                } else {
                    (0.3, "b_preferred", format!("B is more relevant ({:.2} vs {:.2})", score_b, score_a))
                }
            }
            _ => {
                let qa = compute_text_quality(&content_a);
                let qb = compute_text_quality(&content_b);
                if qa >= qb {
                    (0.7, "a_preferred", format!("A preferred overall ({:.2} vs {:.2})", qa, qb))
                } else {
                    (0.3, "b_preferred", format!("B preferred overall ({:.2} vs {:.2})", qb, qa))
                }
            }
        };

        let mut metadata = HashMap::new();
        metadata.insert("input".to_string(), Value::String(input));
        metadata.insert("criterion".to_string(), Value::String(self.criterion.clone()));

        Ok(EvaluationResult::new(score)
            .with_label(label)
            .with_reasoning(reasoning)
            .with_metadata(metadata))
    }
}

fn compute_coherence(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .collect();
    if sentences.len() <= 1 {
        return 0.5;
    }
    let sentence_count = sentences.len() as f64;
    let avg_len = sentences.iter().map(|s| s.split_whitespace().count()).sum::<usize>() as f64 / sentence_count;
    let structural_score = if avg_len > 3.0 && avg_len < 30.0 { 1.0 } else { 0.5 };
    let completeness_score = (sentence_count / 3.0).min(1.0);
    0.5 * structural_score + 0.5 * completeness_score
}

fn compute_relevance(input: &str, response: &str) -> f64 {
    let input_words: std::collections::HashSet<String> = input
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let response_words: std::collections::HashSet<String> = response
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if input_words.is_empty() || response_words.is_empty() {
        return 0.0;
    }

    let overlap = input_words.intersection(&response_words).count() as f64;
    let recall = overlap / input_words.len() as f64;
    recall
}

#[derive(Debug, Clone)]
pub struct SiblingNode {
    pub id: String,
    pub content: String,
    pub score: f64,
}

pub struct EmbeddedSiblingsEvaluator {
    similarity_threshold: f64,
}

impl EmbeddedSiblingsEvaluator {
    pub fn new(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold,
        }
    }
}

impl Default for EmbeddedSiblingsEvaluator {
    fn default() -> Self {
        Self::new(0.7)
    }
}

#[async_trait]
impl Evaluator<String, Vec<SiblingNode>> for EmbeddedSiblingsEvaluator {
    fn name(&self) -> &str {
        "embedded_siblings"
    }

    async fn evaluate(
        &self,
        input: String,
        prediction: Vec<SiblingNode>,
        _reference: Option<Vec<SiblingNode>>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        if prediction.is_empty() {
            return Ok(EvaluationResult::new(0.0)
                .with_label("no_siblings")
                .with_reasoning("No sibling nodes provided"));
        }

        let above_threshold: Vec<&SiblingNode> = prediction
            .iter()
            .filter(|s| s.score >= self.similarity_threshold)
            .collect();

        let ratio = above_threshold.len() as f64 / prediction.len() as f64;

        let max_score = prediction.iter().map(|s| s.score).fold(f64::NEG_INFINITY, f64::max);
        let min_score = prediction.iter().map(|s| s.score).fold(f64::INFINITY, f64::min);
        let avg_score = prediction.iter().map(|s| s.score).sum::<f64>() / prediction.len() as f64;

        let label = if ratio > 0.5 {
            "high_quality"
        } else if ratio > 0.25 {
            "medium_quality"
        } else {
            "low_quality"
        };

        let mut metadata = HashMap::new();
        metadata.insert("input".to_string(), Value::String(input));
        metadata.insert("sibling_count".to_string(), Value::from(prediction.len() as u64));
        metadata.insert("above_threshold".to_string(), Value::from(above_threshold.len() as u64));
        metadata.insert("max_score".to_string(), Value::from(max_score));
        metadata.insert("min_score".to_string(), Value::from(min_score));
        metadata.insert("avg_score".to_string(), Value::Number(serde_json::Number::from_f64(avg_score).unwrap_or_else(|| serde_json::Number::from(0))));

        Ok(EvaluationResult::new(ratio)
            .with_label(label)
            .with_reasoning(format!(
                "{} of {} siblings above threshold {:.2}",
                above_threshold.len(),
                prediction.len(),
                self.similarity_threshold
            ))
            .with_metadata(metadata))
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
