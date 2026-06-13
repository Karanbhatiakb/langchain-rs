//! String evaluation chains — QA, criteria, score, and string distance evaluators.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::traits::{EvaluationResult, Evaluator};

pub struct QAEvalChain {
    criteria: Vec<String>,
}

impl QAEvalChain {
    pub fn new() -> Self {
        Self {
            criteria: vec![
                "correctness".to_string(),
                "relevance".to_string(),
                "completeness".to_string(),
            ],
        }
    }

    pub fn with_criteria(mut self, criteria: Vec<String>) -> Self {
        self.criteria = criteria;
        self
    }
}

impl Default for QAEvalChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Evaluator<(String, String), String> for QAEvalChain {
    fn name(&self) -> &str {
        "qa_eval_chain"
    }

    async fn evaluate(
        &self,
        (question, reference_answer): (String, String),
        prediction: String,
        _reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let mut scores = HashMap::new();

        for criterion in &self.criteria {
            let score = match criterion.as_str() {
                "correctness" => {
                    let pred_lower = prediction.to_lowercase();
                    let ref_lower = reference_answer.to_lowercase();
                    let pred_words: std::collections::HashSet<&str> = pred_lower.split_whitespace().collect();
                    let ref_words: std::collections::HashSet<&str> = ref_lower.split_whitespace().collect();
                    if pred_words.is_empty() || ref_words.is_empty() {
                        0.0
                    } else {
                        let overlap = pred_words.intersection(&ref_words).count() as f64;
                        let recall = overlap / ref_words.len() as f64;
                        let precision = overlap / pred_words.len() as f64;
                        if precision + recall == 0.0 { 0.0 } else { 2.0 * precision * recall / (precision + recall) }
                    }
                }
        "relevance" => {
                let q_lower = question.to_lowercase();
                let p_lower = prediction.to_lowercase();
                let q_words: std::collections::HashSet<&str> = q_lower.split_whitespace().collect();
                let p_words: std::collections::HashSet<&str> = p_lower.split_whitespace().collect();
                    if q_words.is_empty() || p_words.is_empty() {
                        0.0
                    } else {
                        let overlap = q_words.intersection(&p_words).count() as f64;
                        overlap / q_words.len() as f64
                    }
                }
                "completeness" => {
                    let ref_words: Vec<&str> = reference_answer.split_whitespace().collect();
                    let pred_lower = prediction.to_lowercase();
                    let found = ref_words.iter()
                        .filter(|w| pred_lower.contains(&w.to_lowercase()))
                        .count() as f64;
                    if ref_words.is_empty() { 1.0 } else { found / ref_words.len() as f64 }
                }
                _ => 0.5,
            };
            scores.insert(criterion.clone(), score);
        }

        let overall: f64 = scores.values().sum::<f64>() / scores.len() as f64;
        let label = if overall >= 0.7 { "good" } else if overall >= 0.4 { "acceptable" } else { "poor" };

        let mut metadata = HashMap::new();
        metadata.insert("question".to_string(), Value::String(question));
        metadata.insert("reference_answer".to_string(), Value::String(reference_answer));
        metadata.insert("criteria_scores".to_string(), {
            let mut m = serde_json::Map::new();
            for (k, v) in &scores {
                m.insert(k.clone(), Value::Number(serde_json::Number::from_f64(*v).unwrap_or_else(|| serde_json::Number::from(0))));
            }
            Value::Object(m)
        });

        Ok(EvaluationResult::new(overall)
            .with_label(label)
            .with_metadata(metadata))
    }
}

pub struct CriteriaEvalChainV2 {
    criteria: Vec<Criterion>,
}

#[derive(Debug, Clone)]
pub enum Criterion {
    Conciseness,
    Relevance,
    Correctness,
    Harmfulness,
    Coherence,
    Custom(String),
}

impl Criterion {
    fn name(&self) -> &str {
        match self {
            Criterion::Conciseness => "conciseness",
            Criterion::Relevance => "relevance",
            Criterion::Correctness => "correctness",
            Criterion::Harmfulness => "harmfulness",
            Criterion::Coherence => "coherence",
            Criterion::Custom(name) => name.as_str(),
        }
    }
}

impl CriteriaEvalChainV2 {
    pub fn new(criteria: Vec<Criterion>) -> Self {
        Self { criteria }
    }

    pub fn conciseness() -> Self {
        Self::new(vec![Criterion::Conciseness])
    }

    pub fn relevance() -> Self {
        Self::new(vec![Criterion::Relevance])
    }

    pub fn correctness() -> Self {
        Self::new(vec![Criterion::Correctness])
    }

    pub fn harmfulness() -> Self {
        Self::new(vec![Criterion::Harmfulness])
    }

    pub fn all() -> Self {
        Self::new(vec![
            Criterion::Conciseness,
            Criterion::Relevance,
            Criterion::Correctness,
            Criterion::Harmfulness,
            Criterion::Coherence,
        ])
    }
}

#[async_trait]
impl Evaluator<String, String> for CriteriaEvalChainV2 {
    fn name(&self) -> &str {
        "criteria_eval_v2"
    }

    async fn evaluate(
        &self,
        input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let mut scores = HashMap::new();

        for criterion in &self.criteria {
            let score = match criterion {
                Criterion::Conciseness => {
                    let word_count = prediction.split_whitespace().count();
                    if word_count <= 50 { 1.0 } else if word_count <= 100 { 0.7 } else if word_count <= 200 { 0.4 } else { 0.2 }
                }
                Criterion::Relevance => {
                    let input_lower = input.to_lowercase();
                    let pred_lower = prediction.to_lowercase();
                    let input_words: std::collections::HashSet<&str> = input_lower.split_whitespace().collect();
                    let pred_words: std::collections::HashSet<&str> = pred_lower.split_whitespace().collect();
                    if input_words.is_empty() || pred_words.is_empty() {
                        0.0
                    } else {
                        let overlap = input_words.intersection(&pred_words).count() as f64;
                        (overlap / input_words.len() as f64).min(1.0)
                    }
                }
                Criterion::Correctness => {
                    match &reference {
                        Some(ref_ans) => {
                            let pred_lower = prediction.to_lowercase().trim().to_string();
                            let ref_lower = ref_ans.to_lowercase().trim().to_string();
                            if pred_lower == ref_lower { 1.0 } else {
                                let pred_words: std::collections::HashSet<&str> = pred_lower.split_whitespace().collect();
                                let ref_words: std::collections::HashSet<&str> = ref_lower.split_whitespace().collect();
                                if ref_words.is_empty() { 0.0 } else {
                                    pred_words.intersection(&ref_words).count() as f64 / ref_words.len() as f64
                                }
                            }
                        }
                        None => 0.5,
                    }
                }
                Criterion::Harmfulness => {
                    let lower = prediction.to_lowercase();
                    let harmful_patterns = ["harm", "danger", "kill", "attack", "destroy", "weapon", "illegal", "violence"];
                    let found = harmful_patterns.iter().filter(|p| lower.contains(*p)).count();
                    if found == 0 { 0.0 } else if found <= 1 { 0.3 } else if found <= 3 { 0.7 } else { 1.0 }
                }
                Criterion::Coherence => {
                    let sentences: Vec<&str> = prediction.split(|c| c == '.' || c == '!' || c == '?')
                        .filter(|s| !s.trim().is_empty())
                        .collect();
                    if sentences.is_empty() { 0.0 } else {
                        let avg_len = sentences.iter().map(|s| s.split_whitespace().count()).sum::<usize>() as f64 / sentences.len() as f64;
                        if avg_len >= 3.0 && avg_len <= 25.0 { 1.0 } else if avg_len >= 1.0 { 0.5 } else { 0.2 }
                    }
                }
                Criterion::Custom(_) => 0.5,
            };
            scores.insert(criterion.name().to_string(), score);
        }

        let overall: f64 = scores.values().sum::<f64>() / scores.len() as f64;
        let label = if overall >= 0.7 { "pass" } else if overall >= 0.4 { "marginal" } else { "fail" };

        let mut metadata = HashMap::new();
        metadata.insert("input".to_string(), Value::String(input));
        metadata.insert("criteria_scores".to_string(), {
            let mut m = serde_json::Map::new();
            for (k, v) in &scores {
                m.insert(k.clone(), Value::Number(serde_json::Number::from_f64(*v).unwrap_or_else(|| serde_json::Number::from(0))));
            }
            Value::Object(m)
        });

        Ok(EvaluationResult::new(overall)
            .with_label(label)
            .with_metadata(metadata))
    }
}

pub struct ScoreStringEvalChainV2 {
    max_score: f64,
    criteria: Vec<String>,
}

impl ScoreStringEvalChainV2 {
    pub fn new(max_score: f64) -> Self {
        Self {
            max_score,
            criteria: vec!["relevance".to_string(), "completeness".to_string(), "fluency".to_string()],
        }
    }

    pub fn with_criteria(mut self, criteria: Vec<String>) -> Self {
        self.criteria = criteria;
        self
    }
}

impl Default for ScoreStringEvalChainV2 {
    fn default() -> Self {
        Self::new(10.0)
    }
}

#[async_trait]
impl Evaluator<String, String> for ScoreStringEvalChainV2 {
    fn name(&self) -> &str {
        "score_string_v2"
    }

    async fn evaluate(
        &self,
        input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let mut component_scores = HashMap::new();

        for criterion in &self.criteria {
            let raw = match criterion.as_str() {
                "relevance" => {
                    let input_lower = input.to_lowercase();
                    let pred_lower = prediction.to_lowercase();
                    let input_words: std::collections::HashSet<&str> = input_lower.split_whitespace().collect();
                    let pred_words: std::collections::HashSet<&str> = pred_lower.split_whitespace().collect();
                    if input_words.is_empty() || pred_words.is_empty() { 0.5 } else {
                        let overlap = input_words.intersection(&pred_words).count() as f64;
                        (overlap / input_words.len() as f64).min(1.0)
                    }
                }
                "completeness" => {
                    match &reference {
                Some(ref_ans) => {
                        let ref_lower = ref_ans.to_lowercase();
                        let pred_lower = prediction.to_lowercase();
                        let ref_words: std::collections::HashSet<&str> = ref_lower.split_whitespace().collect();
                        let pred_words: std::collections::HashSet<&str> = pred_lower.split_whitespace().collect();
                            if ref_words.is_empty() { 1.0 } else {
                                ref_words.intersection(&pred_words).count() as f64 / ref_words.len() as f64
                            }
                        }
                        None => 0.5,
                    }
                }
                "fluency" => {
                    let words = prediction.split_whitespace().count();
                    if words == 0 { 0.0 } else if words < 3 { 0.3 } else if words < 10 { 0.6 } else { 0.9 }
                }
                _ => 0.5,
            };
            component_scores.insert(criterion.clone(), raw);
        }

        let avg: f64 = component_scores.values().sum::<f64>() / component_scores.len() as f64;
        let score = avg * self.max_score;

        let label = if score >= self.max_score * 0.7 { "good" } else if score >= self.max_score * 0.4 { "fair" } else { "poor" };

        let mut metadata = HashMap::new();
        metadata.insert("input".to_string(), Value::String(input));
        metadata.insert("max_score".to_string(), Value::Number(serde_json::Number::from_f64(self.max_score).unwrap_or_else(|| serde_json::Number::from(0))));
        metadata.insert("component_scores".to_string(), {
            let mut m = serde_json::Map::new();
            for (k, v) in &component_scores {
                m.insert(k.clone(), Value::Number(serde_json::Number::from_f64(*v).unwrap_or_else(|| serde_json::Number::from(0))));
            }
            Value::Object(m)
        });

        Ok(EvaluationResult::new(score)
            .with_label(label)
            .with_metadata(metadata))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StringDistanceType {
    Levenshtein,
    JaroWinkler,
    Hamming,
}

pub struct StringDistanceEvalChain {
    distance_type: StringDistanceType,
    normalize: bool,
}

impl StringDistanceEvalChain {
    pub fn new(distance_type: StringDistanceType) -> Self {
        Self {
            distance_type,
            normalize: true,
        }
    }

    pub fn levenshtein() -> Self {
        Self::new(StringDistanceType::Levenshtein)
    }

    pub fn jaro_winkler() -> Self {
        Self::new(StringDistanceType::JaroWinkler)
    }

    pub fn hamming() -> Self {
        Self::new(StringDistanceType::Hamming)
    }

    pub fn without_normalization(mut self) -> Self {
        self.normalize = false;
        self
    }
}

impl Default for StringDistanceEvalChain {
    fn default() -> Self {
        Self::levenshtein()
    }
}

#[async_trait]
impl Evaluator<String, String> for StringDistanceEvalChain {
    fn name(&self) -> &str {
        match self.distance_type {
            StringDistanceType::Levenshtein => "string_distance_levenshtein",
            StringDistanceType::JaroWinkler => "string_distance_jaro_winkler",
            StringDistanceType::Hamming => "string_distance_hamming",
        }
    }

    async fn evaluate(
        &self,
        _input: String,
        prediction: String,
        reference: Option<String>,
    ) -> Result<EvaluationResult, Box<dyn std::error::Error + Send>> {
        let reference = match reference {
            Some(r) => r,
            None => {
                return Ok(EvaluationResult::new(0.0)
                    .with_label("no_reference")
                    .with_reasoning("No reference provided for string distance evaluation"));
            }
        };

        let (raw_distance, similarity) = match self.distance_type {
            StringDistanceType::Levenshtein => {
                let dist = levenshtein_distance(&prediction, &reference);
                let max_len = prediction.len().max(reference.len());
                let sim = if max_len == 0 { 1.0 } else { 1.0 - dist as f64 / max_len as f64 };
                (dist as f64, sim)
            }
            StringDistanceType::JaroWinkler => {
                let sim = jaro_winkler_similarity(&prediction, &reference);
                let dist = 1.0 - sim;
                (dist, sim)
            }
            StringDistanceType::Hamming => {
                let dist = hamming_distance(&prediction, &reference);
                let max_len = prediction.len().max(reference.len());
                let sim = if max_len == 0 { 1.0 } else { 1.0 - dist as f64 / max_len as f64 };
                (dist as f64, sim)
            }
        };

        let score = if self.normalize { similarity } else { raw_distance };
        let label = if similarity >= 0.8 { "similar" } else if similarity >= 0.5 { "partial" } else { "different" };

        let mut metadata = HashMap::new();
        metadata.insert("distance_type".to_string(), Value::String(self.name().to_string()));
        metadata.insert("raw_distance".to_string(), Value::Number(serde_json::Number::from_f64(raw_distance).unwrap_or_else(|| serde_json::Number::from(0))));
        metadata.insert("similarity".to_string(), Value::Number(serde_json::Number::from_f64(similarity).unwrap_or_else(|| serde_json::Number::from(0))));

        Ok(EvaluationResult::new(score)
            .with_label(label)
            .with_metadata(metadata))
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
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
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

fn jaro_winkler_similarity(s1: &str, s2: &str) -> f64 {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let s1_len = s1_chars.len();
    let s2_len = s2_chars.len();

    if s1_len == 0 && s2_len == 0 {
        return 1.0;
    }
    if s1_len == 0 || s2_len == 0 {
        return 0.0;
    }

    let match_distance = (s1_len.max(s2_len) / 2).saturating_sub(1);
    let mut s1_matches = vec![false; s1_len];
    let mut s2_matches = vec![false; s2_len];

    let mut matches = 0usize;
    let mut transpositions = 0usize;

    for i in 0..s1_len {
        let start = if i > match_distance { i - match_distance } else { 0 };
        let end = (i + match_distance + 1).min(s2_len);

        for j in start..end {
            if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut k = 0usize;
    for i in 0..s1_len {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let jaro = (matches as f64 / s1_len as f64
        + matches as f64 / s2_len as f64
        + (matches - transpositions / 2) as f64 / matches as f64)
        / 3.0;

    let common_prefix_len = s1_chars.iter().zip(s2_chars.iter()).take_while(|(a, b)| a == b).count().min(4);

    jaro + common_prefix_len as f64 * 0.1 * (1.0 - jaro)
}

fn hamming_distance(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let min_len = a_bytes.len().min(b_bytes.len());
    let max_len = a_bytes.len().max(b_bytes.len());

    let mut distance = max_len - min_len;
    for i in 0..min_len {
        if a_bytes[i] != b_bytes[i] {
            distance += 1;
        }
    }
    distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qa_eval_chain_name() {
        let e = QAEvalChain::new();
        assert_eq!(e.name(), "qa_eval_chain");
    }

    #[tokio::test]
    async fn test_qa_eval_chain_default() {
        let e = QAEvalChain::default();
        let result = e.evaluate(
            ("What is Rust?".into(), "A programming language".into()),
            "Rust is a programming language".into(),
            None,
        ).await.unwrap();
        assert!(result.score >= 0.0);
        assert!(result.label.is_some());
    }

    #[tokio::test]
    async fn test_qa_eval_chain_with_criteria() {
        let e = QAEvalChain::new().with_criteria(vec!["correctness".into()]);
        let result = e.evaluate(
            ("Q?".into(), "Answer".into()),
            "Answer".into(),
            None,
        ).await.unwrap();
        assert!(result.score >= 0.0);
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_v2_conciseness() {
        let e = CriteriaEvalChainV2::conciseness();
        assert_eq!(e.name(), "criteria_eval_v2");
        let result = e.evaluate("q".into(), "short".into(), None).await.unwrap();
        assert!(result.score >= 0.5);
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_v2_relevance() {
        let e = CriteriaEvalChainV2::relevance();
        let result = e.evaluate("hello world".into(), "hello world".into(), None).await.unwrap();
        assert!(result.score > 0.0);
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_v2_correctness() {
        let e = CriteriaEvalChainV2::correctness();
        let result = e.evaluate("q".into(), "exact".into(), Some("exact".into())).await.unwrap();
        assert_eq!(result.score, 1.0);
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_v2_harmfulness() {
        let e = CriteriaEvalChainV2::harmfulness();
        let result = e.evaluate("q".into(), "safe content".into(), None).await.unwrap();
        assert_eq!(result.score, 0.0);
    }

    #[tokio::test]
    async fn test_criteria_eval_chain_v2_all() {
        let e = CriteriaEvalChainV2::all();
        assert_eq!(e.criteria.len(), 5);
    }

    #[tokio::test]
    async fn test_score_string_eval_chain_v2() {
        let e = ScoreStringEvalChainV2::new(10.0);
        assert_eq!(e.name(), "score_string_v2");
        let result = e.evaluate("input".into(), "some prediction".into(), None).await.unwrap();
        assert!(result.score >= 0.0);
        assert!(result.score <= 10.0);
    }

    #[tokio::test]
    async fn test_score_string_eval_chain_v2_with_criteria() {
        let e = ScoreStringEvalChainV2::new(5.0)
            .with_criteria(vec!["fluency".into()]);
        let result = e.evaluate("input".into(), "hello world".into(), None).await.unwrap();
        assert!(result.score > 0.0);
    }

    #[tokio::test]
    async fn test_string_distance_levenshtein() {
        let e = StringDistanceEvalChain::levenshtein();
        assert_eq!(e.name(), "string_distance_levenshtein");
        let result = e.evaluate("".into(), "kitten".into(), Some("sitting".into())).await.unwrap();
        assert!(result.score >= 0.0);
    }

    #[tokio::test]
    async fn test_string_distance_jaro_winkler() {
        let e = StringDistanceEvalChain::jaro_winkler();
        let result = e.evaluate("".into(), "hello".into(), Some("hello".into())).await.unwrap();
        assert_eq!(result.score, 1.0);
    }

    #[tokio::test]
    async fn test_string_distance_hamming() {
        let e = StringDistanceEvalChain::hamming();
        let result = e.evaluate("".into(), "abc".into(), Some("abc".into())).await.unwrap();
        assert!((result.score - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_string_distance_no_reference() {
        let e = StringDistanceEvalChain::levenshtein();
        let result = e.evaluate("".into(), "test".into(), None).await.unwrap();
        assert_eq!(result.label.as_deref(), Some("no_reference"));
    }

    #[tokio::test]
    async fn test_string_distance_unnormalized() {
        let e = StringDistanceEvalChain::levenshtein().without_normalization();
        let result = e.evaluate("".into(), "abc".into(), Some("abd".into())).await.unwrap();
        assert!(result.score > 0.0);
    }

    #[tokio::test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_jaro_winkler_similarity() {
        let sim = jaro_winkler_similarity("hello", "hello");
        assert!((sim - 1.0).abs() < 1e-6);
        let sim2 = jaro_winkler_similarity("", "");
        assert!((sim2 - 1.0).abs() < 1e-6);
        let sim3 = jaro_winkler_similarity("abc", "xyz");
        assert!((sim3 - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_hamming_distance() {
        assert_eq!(hamming_distance("abc", "abc"), 0);
        assert_eq!(hamming_distance("abc", "abd"), 1);
        assert_eq!(hamming_distance("abc", "ab"), 1);
    }

    #[test]
    fn test_string_distance_type_debug() {
        let d = StringDistanceType::Levenshtein;
        assert!(format!("{:?}", d).contains("Levenshtein"));
    }

    #[tokio::test]
    async fn test_string_evaluators_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<QAEvalChain>();
        assert_sync::<QAEvalChain>();
        assert_send::<CriteriaEvalChainV2>();
        assert_sync::<CriteriaEvalChainV2>();
        assert_send::<ScoreStringEvalChainV2>();
        assert_sync::<ScoreStringEvalChainV2>();
        assert_send::<StringDistanceEvalChain>();
        assert_sync::<StringDistanceEvalChain>();
    }
}
