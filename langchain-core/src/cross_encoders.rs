//! Cross-encoder models for re-ranking.
//!
//! Provides the [`CrossEncoder`] trait and a [`FakeCrossEncoder`] for testing.

use crate::errors::*;
use async_trait::async_trait;

/// Trait for cross-encoder models that score text pairs.
#[async_trait]
pub trait CrossEncoder: Send + Sync {
    /// Scores each text pair and returns a list of relevance scores.
    async fn score(&self, text_pairs: Vec<(String, String)>) -> Result<Vec<f32>>;
}

/// A fake cross-encoder that returns deterministic scores based on string
/// length ratio.
///
/// The score for each pair `(a, b)` is `min(a.len(), b.len()) as f32 /
/// max(a.len(), b.len()) as f32`, yielding a value in `[0.0, 1.0]`.
#[derive(Debug, Clone)]
pub struct FakeCrossEncoder;

impl FakeCrossEncoder {
    /// Creates a new `FakeCrossEncoder`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FakeCrossEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CrossEncoder for FakeCrossEncoder {
    async fn score(&self, text_pairs: Vec<(String, String)>) -> Result<Vec<f32>> {
        let scores = text_pairs
            .into_iter()
            .map(|(a, b)| {
                let a_len = a.len() as f32;
                let b_len = b.len() as f32;
                if a_len == 0.0 && b_len == 0.0 {
                    0.0
                } else {
                    a_len.min(b_len) / a_len.max(b_len)
                }
            })
            .collect();
        Ok(scores)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_cross_encoder() {
        let encoder = FakeCrossEncoder::new();
        let pairs = vec![
            ("hello".to_string(), "hello".to_string()),
            ("short".to_string(), "a much longer string here".to_string()),
            ("".to_string(), "".to_string()),
        ];
        let scores = encoder.score(pairs).await.unwrap();
        assert_eq!(scores.len(), 3);
        assert!((scores[0] - 1.0).abs() < f32::EPSILON);
        assert!(scores[1] > 0.0 && scores[1] < 1.0);
        assert!((scores[2] - 0.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn test_fake_cross_encoder_empty() {
        let encoder = FakeCrossEncoder::new();
        let scores = encoder.score(vec![]).await.unwrap();
        assert!(scores.is_empty());
    }

    #[tokio::test]
    async fn test_fake_cross_encoder_identical() {
        let encoder = FakeCrossEncoder::default();
        let pairs = vec![
            ("hello".into(), "hello".into()),
            ("rust".into(), "rust".into()),
        ];
        let scores = encoder.score(pairs).await.unwrap();
        assert_eq!(scores.len(), 2);
        for score in scores {
            assert!((score - 1.0).abs() < f32::EPSILON);
        }
    }

    #[tokio::test]
    async fn test_fake_cross_encoder_empty_one_side() {
        let encoder = FakeCrossEncoder::new();
        let pairs = vec![
            ("".into(), "nonempty".into()),
            ("nonempty".into(), "".into()),
        ];
        let scores = encoder.score(pairs).await.unwrap();
        assert_eq!(scores.len(), 2);
        assert!((scores[0] - 0.0).abs() < f32::EPSILON);
        assert!((scores[1] - 0.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn test_fake_cross_encoder_different_lengths() {
        let encoder = FakeCrossEncoder::new();
        let pairs = vec![
            ("a".into(), "aaaa".into()),
        ];
        let scores = encoder.score(pairs).await.unwrap();
        assert!((scores[0] - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fake_cross_encoder_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FakeCrossEncoder>();
        assert_sync::<FakeCrossEncoder>();
    }

    #[test]
    fn test_fake_cross_encoder_debug_clone() {
        let encoder = FakeCrossEncoder::new();
        let _debug = format!("{:?}", encoder);
        let _cloned = encoder.clone();
    }

    #[tokio::test]
    async fn test_fake_cross_encoder_zero_length_a() {
        let encoder = FakeCrossEncoder::new();
        let scores = encoder.score(vec![("".into(), "abc".into())]).await.unwrap();
        assert!((scores[0] - 0.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn test_fake_cross_encoder_zero_length_b() {
        let encoder = FakeCrossEncoder::new();
        let scores = encoder.score(vec![("abc".into(), "".into())]).await.unwrap();
        assert!((scores[0] - 0.0).abs() < f32::EPSILON);
    }
}
