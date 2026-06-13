use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use langchain_core::documents::Document;
use langchain_core::messages::{BaseMessage, HumanMessage, MessageType};
use langchain_core::output_parsers::{JsonOutputParser, OutputParser, StrOutputParser};
use langchain_core::prompt::PromptTemplate;
use std::collections::HashMap;

fn bench_prompt_template_from_template(c: &mut Criterion) {
    let mut group = c.benchmark_group("prompt_template_from_template");
    let templates = [
        ("simple", "Hello {name}!"),
        ("medium", "You are a {role}. Your task is to {task}. Please be {style}."),
        (
            "complex",
            "Given the context: {context}\nQuestion: {question}\nFormat: {format}\nLanguage: {language}\nTone: {tone}",
        ),
    ];
    for (label, template) in &templates {
        group.bench_with_input(BenchmarkId::new("parse", label), template, |b, t| {
            b.iter(|| PromptTemplate::from_template(t));
        });
    }
    group.finish();
}

fn bench_prompt_template_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("prompt_template_format");
    let template = PromptTemplate::from_template(
        "You are a {role}. Your task is to {task}. Please respond in {language}.",
    );
    let mut kwargs = HashMap::new();
    kwargs.insert("role".to_string(), "assistant".to_string());
    kwargs.insert("task".to_string(), "answer questions".to_string());
    kwargs.insert("language".to_string(), "English".to_string());

    group.throughput(Throughput::Elements(1));
    group.bench_function("3_vars", |b| {
        b.iter(|| template.format(&kwargs).unwrap());
    });

    let long_template = PromptTemplate::from_template(&format!(
        "System: {{system}}\nContext: {{context}}\nQuestion: {{question}}\n{}\nAnswer: {{answer}}",
        "Additional info: {info}".repeat(10)
    ));
    let mut long_kwargs = HashMap::new();
    long_kwargs.insert("system".to_string(), "You are helpful.".to_string());
    long_kwargs.insert("context".to_string(), "Some context here.".to_string());
    long_kwargs.insert("question".to_string(), "What is Rust?".to_string());
    long_kwargs.insert("answer".to_string(), "A systems language.".to_string());
    long_kwargs.insert("info".to_string(), "Extra info.".to_string());
    group.bench_function("long_template", |b| {
        b.iter(|| long_template.format(&long_kwargs).unwrap());
    });
    group.finish();
}

fn bench_str_output_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("str_output_parser");
    let parser = StrOutputParser;
    let text = "This is a sample output from an LLM that contains multiple sentences. It has some complexity.";
    group.bench_function("parse", |b| {
        b.iter(|| parser.parse(text).unwrap());
    });
    group.finish();
}

fn bench_json_output_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_output_parser");
    let parser = JsonOutputParser::<serde_json::Value>::new();

    let simple_json = r#"{"answer": "42", "confidence": 0.95}"#;
    group.bench_function("simple_object", |b| {
        b.iter(|| parser.parse(simple_json).unwrap());
    });

    let nested_json = r#"{"result": {"answer": "Paris", "sources": ["a", "b", "c"], "metadata": {"model": "gpt-4", "tokens": 150}}}"#;
    group.bench_function("nested_object", |b| {
        b.iter(|| parser.parse(nested_json).unwrap());
    });
    group.finish();
}

fn bench_document_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_creation");
    group.bench_function("new_simple", |b| {
        b.iter(|| Document::new("This is a sample document with some text content."));
    });

    group.bench_function("new_with_metadata", |b| {
        b.iter(|| {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), serde_json::Value::String("web".to_string()));
            metadata.insert("page".to_string(), serde_json::Value::Number(42.into()));
            Document::new("This is a sample document with some text content.").with_metadata(metadata)
        });
    });

    group.bench_function("new_with_score", |b| {
        b.iter(|| {
            Document::new("This is a sample document with some text content.").with_score(0.95)
        });
    });
    group.finish();
}

fn bench_message_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_creation");
    group.bench_function("base_message_new", |b| {
        b.iter(|| BaseMessage::new("Hello, how can I help you?", MessageType::Human));
    });

    group.bench_function("human_message_new", |b| {
        b.iter(|| HumanMessage::new("What is the capital of France?"));
    });

    group.bench_function("base_message_with_name", |b| {
        b.iter(|| {
            BaseMessage::new("Hello", MessageType::Human).with_name("user_1")
        });
    });

    group.bench_function("base_message_with_metadata", |b| {
        b.iter(|| {
            let mut metadata = HashMap::new();
            metadata.insert("session_id".to_string(), serde_json::Value::String("abc123".to_string()));
            BaseMessage::new("Hello", MessageType::Human).with_metadata(metadata)
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_prompt_template_from_template,
    bench_prompt_template_format,
    bench_str_output_parser,
    bench_json_output_parser,
    bench_document_creation,
    bench_message_creation,
);
criterion_main!(benches);
