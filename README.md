# langchain-rs

An experimental Rust implementation of the LangChain ecosystem. The goal of this project is to bring the modular LLM orchestration abstractions of LangChain to Rust, leveraging the language's thread safety, strict compile-time checks, and async runtime.

## Why Rust for LangChain?

While Python is excellent for rapid prototyping, production LLM systems benefit greatly from Rust's performance and safety guarantees:
* **Strong Type Safety**: Prevent runtime bugs (like passing mismatched inputs between chains/tools) at compile time.
* **Concurrency by Default**: Implement asynchronous agent loops, concurrent tool execution, and stream processing without threading issues or GIL bottlenecks.
* **Low Overhead**: Zero-cost abstractions and fast startup times make it ideal for resource-constrained environments or high-throughput API servers.

---

## Workspace Structure

The project is organized as a Cargo workspace containing modular crates:

* `langchain-core`: Core traits like `Runnable`, `Callbacks`, `Message`, and `PromptTemplate`.
* `langchain-llms`: Integration wrapper types for LLM providers (including OpenAI, Anthropic, Google Gemini, and test fakes).
* `langchain-embeddings`: Interface and model wrappers for text embeddings.
* `langchain-vectorstores`: Vector store implementations (including an in-memory db).
* `langchain-tools`: Utility tools (e.g., a math expression calculator and shell execution tool).
* `langchain-document-loaders`: Document parsing (Text, CSV, etc.).
* `langchain-memory`: Chat history and context buffers.
* `langchain-chains`: Assembled runnables (like sequential chains, LLMChain, and QA chains).
* `langchain-agents`: Executable agent patterns (like ReAct agent executors).
* `langchain-serve`: Axum-based server framework for exposing chains via REST endpoints (similar to LangServe).
* `langchain-examples`: Demos and example implementations.

---

## Quick Start

Make sure you have Rust and Cargo installed. To run the examples suite:

```bash
# Run all the demos
cargo run --package langchain-examples --bin langchain-examples
```

### Basic LCEL (LangChain Expression Language) Sequence Example

You can chain runnable functions sequentially using a pipe-like sequence:

```rust
use langchain_core::errors::Result;
use langchain_core::runnable::Runnable;
use serde_json::Value;

// Simple custom runnable
struct UpperCaseRunnable;

#[async_trait::async_trait]
impl Runnable<Value, Value> for UpperCaseRunnable {
    async fn invoke(&self, input: Value) -> Result<Value> {
        match input {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            other => Ok(other),
        }
    }
}
```

For more comprehensive usage (agents, memory, prompt templates, output parsers), check the files under `langchain-examples/src/`.

---

## Current Status & Roadmap

This project is currently in active development:
* **Core abstractions** (Runnable, Prompts, Callbacks, Chains) are implemented and tested.
* **LLM providers** include full implementations for OpenAI and local fakes. Other provider configurations are being added.
* **Text splitters & Tokenizers** support character, token (Cl100K), and markdown splitting.
* **LangServe** is currently in an early preview stage with basic route stubs.

Contributions, issues, and ideas are welcome.

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
