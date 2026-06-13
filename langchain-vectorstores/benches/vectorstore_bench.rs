use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use langchain_vectorstores::utils::{cosine_similarity, normalize_vector, top_k_by_score};

fn bench_cosine_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosine_similarity");

    let dims = [8, 64, 256, 1024];
    for &dim in &dims {
        let va: Vec<f32> = (0..dim).map(|i| (i as f32).sin()).collect();
        let vb: Vec<f32> = (0..dim).map(|i| (i as f32).cos()).collect();
        group.bench_with_input(BenchmarkId::new("dim", dim), &(&va, &vb), |bencher, (a, b)| {
            bencher.iter(|| cosine_similarity(a, b));
        });
    }

    let identical: Vec<f32> = (0..256).map(|i| (i as f32).sin()).collect();
    group.bench_function("identical_256", |b| {
        b.iter(|| cosine_similarity(&identical, &identical));
    });

    let orthogonal_a: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0];
    let orthogonal_b: Vec<f32> = vec![0.0, 1.0, 0.0, 0.0];
    group.bench_function("orthogonal_4", |b| {
        b.iter(|| cosine_similarity(&orthogonal_a, &orthogonal_b));
    });
    group.finish();
}

fn bench_normalize_vector(c: &mut Criterion) {
    let mut group = c.benchmark_group("normalize_vector");

    let dims = [8, 64, 256, 1024];
    for &dim in &dims {
        group.bench_with_input(BenchmarkId::new("dim", dim), &dim, |b, &dim| {
            b.iter_batched(
                || (0..dim).map(|i| (i as f32).sin()).collect::<Vec<f32>>(),
                |mut v| normalize_vector(&mut v),
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_top_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("top_k");

    for &n in &[10usize, 100, 1000, 10000] {
        let scores: Vec<(usize, f32)> = (0..n)
            .map(|i| (i, ((i as f32 * 0.123).sin() + 1.0) / 2.0))
            .collect();

        for &k in &[1, 5, 10] {
            if k > n {
                continue;
            }
            group.bench_with_input(
                BenchmarkId::new(format!("n_{}", n), k),
                &(scores.clone(), k),
                |b, (scores, k)| {
                    b.iter(|| top_k_by_score(scores.clone(), *k));
                },
            );
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_cosine_similarity,
    bench_normalize_vector,
    bench_top_k,
);
criterion_main!(benches);
