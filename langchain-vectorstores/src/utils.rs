//! Vector utility functions for similarity computation, normalization, and MMR.

/// Computes cosine similarity between two equal-length float slices.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

/// Computes Euclidean distance between two float slices.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

/// Selects the indices of the top-k scores (descending order).
pub fn top_k_by_score(mut scores: Vec<(usize, f32)>, k: usize) -> Vec<usize> {
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores.into_iter().take(k).map(|(idx, _)| idx).collect()
}

/// Normalizes a vector in place to unit length.
pub fn normalize_vector(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in v.iter_mut() {
            *val /= norm;
        }
    }
}

/// Computes Max Marginal Relevance (MMR) scores to select a diverse subset of
/// embeddings.
///
/// MMR balances relevance to the query (`lambda_mult`) against diversity among
/// the selected items.
pub fn max_marginal_relevance(
    query_embedding: &[f32],
    embedding_list: &[Vec<f32>],
    k: usize,
    lambda_mult: f32,
) -> Vec<usize> {
    if embedding_list.is_empty() || k == 0 {
        return Vec::new();
    }

    let mut normalized_query = query_embedding.to_vec();
    normalize_vector(&mut normalized_query);

    let mut normalized_embeddings: Vec<Vec<f32>> = embedding_list.to_vec();
    for emb in &mut normalized_embeddings {
        normalize_vector(emb);
    }

    let query_dot_product: Vec<f32> = normalized_embeddings
        .iter()
        .map(|emb| cosine_similarity(&normalized_query, emb))
        .collect();

    mmr_helper(&query_dot_product, &normalized_embeddings, k, lambda_mult)
}

/// Internal helper implementing the MMR selection loop.
fn mmr_helper(
    query_scores: &[f32],
    embeddings: &[Vec<f32>],
    k: usize,
    lambda_mult: f32,
) -> Vec<usize> {
    let mut selected: Vec<usize> = Vec::new();
    let mut candidate_indices: Vec<usize> = (0..embeddings.len()).collect();

    if k >= embeddings.len() {
        let mut sorted: Vec<(usize, f32)> = query_scores
            .iter()
            .enumerate()
            .map(|(i, s)| (i, *s))
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        return sorted.into_iter().map(|(i, _)| i).collect();
    }

    if let Some((idx, _)) = query_scores
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
    {
        selected.push(idx);
        candidate_indices.retain(|&i| i != idx);
    }

    while selected.len() < k && !candidate_indices.is_empty() {
        let mut mmr_scores: Vec<(usize, f32)> = candidate_indices
            .iter()
            .map(|&i| {
                let sim_to_query = query_scores[i];
                let max_sim_to_selected = selected
                    .iter()
                    .map(|&s| cosine_similarity(&embeddings[i], &embeddings[s]))
                    .fold(f32::NEG_INFINITY, f32::max);
                let mmr = lambda_mult * sim_to_query - (1.0 - lambda_mult) * max_sim_to_selected;
                (i, mmr)
            })
            .collect();

        mmr_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((next_idx, _)) = mmr_scores.first() {
            let next_idx = *next_idx;
            selected.push(next_idx);
            candidate_indices.retain(|&i| i != next_idx);
        }
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);

        let c = vec![1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_top_k_by_score() {
        let scores = vec![(0, 0.5), (1, 0.9), (2, 0.3), (3, 0.7)];
        let top = top_k_by_score(scores, 2);
        assert_eq!(top, vec![1, 3]);
    }

    #[test]
    fn test_normalize_vector() {
        let mut v = vec![3.0, 4.0];
        normalize_vector(&mut v);
        assert!((v.iter().map(|x| x * x).sum::<f32>().sqrt() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_zero_vectors() {
        let a = vec![0.0, 0.0];
        let b = vec![0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_partial() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.5, 0.5, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_euclidean_distance_identical() {
        let a = vec![1.0, 2.0, 3.0];
        assert!((euclidean_distance(&a, &a) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_top_k_by_score_empty() {
        let top = top_k_by_score(vec![], 5);
        assert!(top.is_empty());
    }

    #[test]
    fn test_top_k_by_score_more_than_exist() {
        let scores = vec![(0, 1.0), (1, 0.5)];
        let top = top_k_by_score(scores, 10);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_normalize_vector_already_unit() {
        let mut v = vec![1.0, 0.0];
        normalize_vector(&mut v);
        assert!((v[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_vector_zero() {
        let mut v = vec![0.0, 0.0];
        normalize_vector(&mut v);
        assert_eq!(v, vec![0.0, 0.0]);
    }

    #[test]
    fn test_max_marginal_relevance_empty() {
        let result = max_marginal_relevance(&[1.0, 0.0], &[], 5, 0.5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_max_marginal_relevance_k_zero() {
        let embeddings = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let result = max_marginal_relevance(&[1.0, 0.0], &embeddings, 0, 0.5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_max_marginal_relevance_single() {
        let embeddings = vec![vec![1.0, 0.0]];
        let result = max_marginal_relevance(&[1.0, 0.0], &embeddings, 1, 0.5);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_utils_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<[f32; 3]>();
        assert_sync::<[f32; 3]>();
    }
}
