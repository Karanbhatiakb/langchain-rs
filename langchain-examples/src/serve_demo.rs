//! serve_demo module.

pub async fn run() -> anyhow::Result<()> {
    println!("LangServe demo showing server setup (not starting server in test mode):");

    let mut server = langchain_serve::server::LangServe::new()
        .with_port(8080)
        .with_cors(true);

    server.add_chain(
        "chat",
        "ChatChain",
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"},
                "config": {"type": "object"}
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "output": {"type": "string"},
                "execution_time_ms": {"type": "integer"}
            }
        }),
        Some("A simple chat chain".into()),
    );

    server.add_chain(
        "qa",
        "QAChain",
        serde_json::json!({
            "type": "object",
            "properties": {
                "question": {"type": "string"},
                "context": {"type": "string"}
            }
        }),
        serde_json::json!({
            "type": "object",
            "properties": {
                "answer": {"type": "string"}
            }
        }),
        Some("Question-answering chain".into()),
    );

    println!("  Server configured with {} chains", 2);
    println!("  Endpoints:");
    println!("    POST /chat/invoke");
    println!("    POST /qa/invoke");
    println!("    GET  /health");
    println!("    GET  /docs");
    println!("\n  To start: call server.start().await");

    let invoke_req = langchain_serve::schemas::InvokeRequest {
        input: serde_json::json!({"message": "Hello"}),
        config: None,
    };
    println!("\n  Sample invoke request: {:?}", serde_json::to_string(&invoke_req).unwrap_or_default());

    Ok(())
}
