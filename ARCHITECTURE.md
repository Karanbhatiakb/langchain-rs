# Architecture

## Design Decisions
- Every trait is Send + Sync + 'static for thread safety
- All async methods use async-trait
- Arc<dyn Trait> for runtime polymorphism
- RwLock/Mutex (parking_lot) for interior mutability
- serde_json::Value for flexible metadata
- thiserror for error enums
- Builder pattern for complex structs
- Feature flags for optional integrations

## Python-to-Rust Mapping
| Python | Rust |
|--------|------|
| BaseMessage | BaseMessage enum variant |
| Runnable | Runnable<I, O> trait |
| Chain | Chain trait (or Runnable) |
| BaseMemory | BaseMemory trait |
| BaseTool | BaseTool trait |
| Agent | Agent trait |
| VectorStore | VectorStore trait |
| Embeddings | Embeddings trait |
| TextSplitter | TextSplitter trait |
| BaseLoader | BaseLoader trait |
| CallbackHandler | CallbackHandler trait |
| OutputParser | OutputParser<T> trait |
| PromptTemplate | PromptTemplate struct |
