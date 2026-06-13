use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use langchain_tools::calculator::CalculatorTool;
use langchain_tools::traits::BaseTool;

fn bench_calculator_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculator_arithmetic");
    let calc = CalculatorTool;
    let rt = tokio::runtime::Runtime::new().unwrap();

    let expressions = [
        ("addition", "2 + 3"),
        ("multiplication", "12 * 7"),
        ("division", "144 / 12"),
        ("power", "2 ^ 10"),
        ("complex", "(3 + 5) * (10 - 2) / 4"),
        ("unary", "-42 + 100"),
    ];

    for (label, expr) in &expressions {
        group.bench_with_input(BenchmarkId::new("invoke", label), expr, |b, e| {
            b.iter(|| rt.block_on(calc.invoke(e)));
        });
    }
    group.finish();
}

fn bench_calculator_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculator_functions");
    let calc = CalculatorTool;
    let rt = tokio::runtime::Runtime::new().unwrap();

    let funcs = [
        ("sqrt", "sqrt(144)"),
        ("sin", "sin(1.5708)"),
        ("cos", "cos(0)"),
        ("log", "log(100)"),
        ("ln", "ln(2.7183)"),
        ("nested", "sqrt(sin(0.5) + cos(0.5))"),
    ];

    for (label, expr) in &funcs {
        group.bench_with_input(BenchmarkId::new("invoke", label), expr, |b, e| {
            b.iter(|| rt.block_on(calc.invoke(e)));
        });
    }
    group.finish();
}

fn bench_calculator_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculator_chain");
    let calc = CalculatorTool;
    let rt = tokio::runtime::Runtime::new().unwrap();

    group.throughput(Throughput::Elements(5));
    group.bench_function("5_ops_sequence", |b| {
        b.iter(|| {
            let results: Vec<String> = ["1 + 1", "2 * 3", "6 / 2", "3 ^ 2", "sqrt(81)"]
                .iter()
                .map(|e| rt.block_on(calc.invoke(e)).unwrap())
                .collect();
            results
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_calculator_arithmetic,
    bench_calculator_functions,
    bench_calculator_chain,
);
criterion_main!(benches);
