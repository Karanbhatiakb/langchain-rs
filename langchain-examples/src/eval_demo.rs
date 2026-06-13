//! eval_demo module.

use langchain_evaluation::datasets::{Dataset, DatasetExample};
use langchain_evaluation::metrics::{
    ContainsMetric, ExactMatchMetric, F1ScoreMetric, StringDistanceMetric,
};
use langchain_evaluation::runners::run_evaluation;

pub async fn run() -> anyhow::Result<()> {
    let mut dataset = Dataset::new("greetings");
    dataset.add_example(DatasetExample::new(
        serde_json::json!("hello"),
        serde_json::json!("hi there"),
    ));
    dataset.add_example(DatasetExample::new(
        serde_json::json!("goodbye"),
        serde_json::json!("see you later"),
    ));

    let exact = ExactMatchMetric;
    let report = run_evaluation(
        &exact,
        &dataset,
        |input| input.as_str().unwrap_or("").to_string(),
        |output| output.as_str().unwrap_or("").to_string(),
        |_output| None::<String>,
    )
    .await;
    println!(
        "ExactMatch: mean={:.2} pass_rate={:.2}",
        report.mean_score, report.pass_rate
    );

    let contains = ContainsMetric::new();
    let report = run_evaluation(
        &contains,
        &dataset,
        |input| input.as_str().unwrap_or("").to_string(),
        |output| output.as_str().unwrap_or("").to_string(),
        |_output| None::<String>,
    )
    .await;
    println!(
        "Contains: mean={:.2} pass_rate={:.2}",
        report.mean_score, report.pass_rate
    );

    let f1 = F1ScoreMetric;
    let report = run_evaluation(
        &f1,
        &dataset,
        |input| input.as_str().unwrap_or("").to_string(),
        |output| output.as_str().unwrap_or("").to_string(),
        |output| Some(output.as_str().unwrap_or("").to_string()),
    )
    .await;
    println!("F1: mean={:.2} pass_rate={:.2}", report.mean_score, report.pass_rate);

    let lev = StringDistanceMetric;
    let report = run_evaluation(
        &lev,
        &dataset,
        |input| input.as_str().unwrap_or("").to_string(),
        |output| output.as_str().unwrap_or("").to_string(),
        |_output| None::<String>,
    )
    .await;
    println!(
        "Levenshtein: mean={:.2} pass_rate={:.2}",
        report.mean_score, report.pass_rate
    );

    Ok(())
}
