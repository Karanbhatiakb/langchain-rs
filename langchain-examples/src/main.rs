//! main module.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use langchain_chains::conversational::ConversationalRetrievalChain;
use langchain_chains::llm_chain::LLMChain;
use langchain_chains::retrieval::RetrievalQA;
use langchain_chains::sequential::SimpleSequentialChain;
use langchain_chains::transform::TransformChain;
use langchain_chains::types::Chain;
use langchain_core::documents::Document;
use langchain_core::messages::HumanMessage;
use langchain_core::output_parsers::{
    CommaSeparatedListOutputParser, JsonOutputParser, OutputParser, StrOutputParser,
};
use langchain_core::prompt::{ChatPromptTemplate, MessageTemplate, PromptTemplate};
use langchain_core::runnable::Runnable;
use langchain_core::text_splitters::{RecursiveCharacterTextSplitter, TextSplitter};
use langchain_llms::fake::FakeListLLM;
use langchain_llms::traits::ChatModel;
use langchain_memory::traits::BaseMemory;
use serde_json::Value;

mod callbacks_demo;
mod eval_demo;
mod lcel_demo;
mod serve_demo;
mod streaming_demo;
mod vector_store_demo;

macro_rules! section {
    ($title:expr) => {
        println!("\n{}", "=".repeat(60));
        println!("  {}", $title);
        println!("{}", "=".repeat(60));
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    section!("1. Basic Chat — LLMChain with FakeListLLM");
    basic_chat().await?;

    section!("2. ReAct Agent");
    agent_react().await?;

    section!("3. Conversational Memory");
    conversational_memory().await?;

    section!("4. Output Parsers");
    output_parsers()?;

    section!("5. Text Splitters");
    text_splitters()?;

    section!("6. Retrieval QA");
    retriever_qa().await?;

    section!("7. Sequential Chain + Transform Chain");
    sequential_chain().await?;

    section!("8. Tools Demo (Calculator + Shell)");
    tools_demo().await?;

    section!("9. Document Loaders");
    document_loaders().await?;

    section!("10. Chat Prompt Templates");
    chat_prompts()?;

    section!("11. Streaming Demo");
    streaming_demo::run().await?;

    section!("12. Evaluation Demo");
    eval_demo::run().await?;

    section!("13. LCEL Composition Demo");
    lcel_demo::run().await?;

    section!("14. Callbacks Demo");
    callbacks_demo::run().await?;

    section!("15. Vector Store Demo");
    vector_store_demo::run().await?;

    section!("16. LangServe Demo");
    serve_demo::run().await?;

    section!("17. Conversational Retrieval Chain");
    conversational_retrieval_demo().await?;

    println!("\n{}", "=".repeat(60));
    println!("  All examples completed successfully!");
    println!("{}", "=".repeat(60));

    Ok(())
}

async fn basic_chat() -> Result<()> {
    let llm = Arc::new(FakeListLLM::new(vec![
        "Rust is a systems programming language focused on safety and performance.".into(),
    ]));
    let prompt = PromptTemplate::from_template("Tell me about {topic}");
    let chain: LLMChain<String> = LLMChain::new(llm, prompt);
    let result = chain
        .invoke(HashMap::from([(
            "topic".into(),
            Value::String("Rust programming".into()),
        )]))
        .await?;
    println!("Result: {}", result);
    Ok(())
}

async fn agent_react() -> Result<()> {
    use langchain_agents::executor::AgentExecutor;
    use langchain_agents::react::ReActAgent;
    
    use langchain_tools::traits::BaseTool;
    use langchain_tools::utility::CalculatorTool;

    let llm = Arc::new(FakeListLLM::new(vec![
        "Thought: I need to calculate\nAction: calculator\nAction Input: 2+2\nObservation: 4\nFinal Answer: The answer is 4".into(),
    ]));
    let tools: Vec<Arc<dyn BaseTool>> = vec![Arc::new(CalculatorTool)];
    let agent = ReActAgent::new(llm.clone(), tools.clone(), None);
    let executor = AgentExecutor::new(Arc::new(agent), tools);
    let result = executor
        .run(&HashMap::from([(
            "input".into(),
            Value::String("What is 2+2?".into()),
        )]))
        .await?;
    println!("Result: {}", result);
    Ok(())
}

async fn conversational_memory() -> Result<()> {
    use langchain_memory::buffer::ConversationBufferMemory as BufferMemory;

    let memory = Arc::new(BufferMemory::new().with_return_messages(false));
    let llm = Arc::new(FakeListLLM::new(vec![
        "Hello! How can I help you today?".into(),
        "I'm doing great!".into(),
        "I enjoy programming in Rust!".into(),
    ]));
    let prompt = PromptTemplate::from_template("The following is a conversation:\n{history}\nHuman: {input}\nAI:");

    for input in &["Hi there!", "How are you?", "What do you do?"] {
        let vars = memory.load_memory_variables(&HashMap::new()).await?;
        let chat_history = vars.get("history").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut format_inputs = HashMap::new();
        format_inputs.insert("input".to_string(), input.to_string());
        format_inputs.insert("history".to_string(), chat_history);
        let formatted = prompt.format(&format_inputs)?;
        let msg = llm.predict_messages(&[HumanMessage::new(&formatted).into()], None, None).await?;
        let output = msg.content;
        memory.save_context(
            &HashMap::from([("input".into(), Value::String(input.to_string()))]),
            &HashMap::from([("output".into(), Value::String(output.clone()))]),
        ).await?;
        println!("Human: {} | AI: {}", input, output);
    }
    Ok(())
}

fn output_parsers() -> Result<()> {
    let str_parser = StrOutputParser;
    println!("StrOutputParser: {}", str_parser.parse("hello")?);

    let json_parser: JsonOutputParser<Value> = JsonOutputParser::new();
    println!("JsonOutputParser: {}", json_parser.parse(r#"{"name":"test"}"#)?);

    let list_parser = CommaSeparatedListOutputParser;
    println!("CommaSeparatedListOutputParser: {:?}", list_parser.parse("a,b,c")?);
    Ok(())
}

fn text_splitters() -> Result<()> {
    let doc = "This is a long document. It has multiple sentences.\n\nIt also has multiple paragraphs.\n\nTesting text splitters.";
    let rec_splitter = RecursiveCharacterTextSplitter::new(
        vec!["\n\n".into(), "\n".into(), " ".into(), "".into()],
        50,
        10,
    );
    let chunks = rec_splitter.split_text(doc)?;
    println!("RecursiveCharacterTextSplitter ({} chunks): {:?}", chunks.len(), chunks);
    Ok(())
}

async fn retriever_qa() -> Result<()> {
    use langchain_core::retrievers::BaseRetriever;
    use langchain_vectorstores::traits::VectorStore;

    #[allow(dead_code)]
    struct SimpleRetriever { documents: Vec<Document> }
    #[async_trait::async_trait]
    impl BaseRetriever for SimpleRetriever {
        async fn get_relevant_documents(&self, _query: &str) -> langchain_core::errors::Result<Vec<Document>> {
            Ok(self.documents.clone())
        }
        async fn add_documents(&self, _documents: Vec<Document>) -> langchain_core::errors::Result<()> {
            Ok(())
        }
    }

    struct DummyVectorStore;
    #[async_trait::async_trait]
    impl VectorStore for DummyVectorStore {
        async fn add_documents(&self, _docs: Vec<Document>) -> langchain_core::errors::Result<Vec<String>> {
            Ok(vec![])
        }
        async fn add_texts(&self, _texts: Vec<String>, _metadatas: Option<Vec<HashMap<String, Value>>>) -> langchain_core::errors::Result<Vec<String>> {
            Ok(vec![])
        }
        async fn similarity_search(&self, _query: &str, _k: usize) -> langchain_core::errors::Result<Vec<Document>> {
            Ok(vec![])
        }
        async fn similarity_search_with_score(&self, _query: &str, _k: usize) -> langchain_core::errors::Result<Vec<(Document, f32)>> {
            Ok(vec![])
        }
        async fn similarity_search_by_vector(&self, _embedding: Vec<f32>, _k: usize) -> langchain_core::errors::Result<Vec<Document>> {
            Ok(vec![])
        }
        async fn max_marginal_relevance_search(&self, _query: &str, _k: usize, _fetch_k: usize, _lambda_mult: f32) -> langchain_core::errors::Result<Vec<Document>> {
            Ok(vec![])
        }
        async fn delete(&self, _ids: Vec<String>) -> langchain_core::errors::Result<()> {
            Ok(())
        }
        fn embeddings(&self) -> Arc<dyn langchain_embeddings::traits::Embeddings> {
            panic!("DummyVectorStore does not provide embeddings")
        }
    }

    let llm = Arc::new(FakeListLLM::new(vec!["Based on the context, Rust is a systems programming language.".into()]));
    let vectorstore = Arc::new(DummyVectorStore);
    let chain = RetrievalQA::new(llm, vectorstore);
    let result = chain.call(HashMap::from([("query".into(), Value::String("What is Rust?".into()))])).await?;
    println!("QA Result: {}", result.get("result").and_then(|v| v.as_str()).unwrap_or(""));
    Ok(())
}

async fn sequential_chain() -> Result<()> {
    let preprocess = Arc::new(TransformChain::new(
        vec!["input".into()],
        vec!["transformed".into()],
        Arc::new(|input: HashMap<String, Value>| -> langchain_core::errors::Result<HashMap<String, Value>> {
            let text = input.get("input").and_then(|v| v.as_str()).unwrap_or("");
            Ok(HashMap::from([("transformed".into(), Value::String(text.to_uppercase()))]))
        }),
    ));
    let postprocess = Arc::new(TransformChain::new(
        vec!["transformed".into()],
        vec!["output".into()],
        Arc::new(|input: HashMap<String, Value>| -> langchain_core::errors::Result<HashMap<String, Value>> {
            let text = input.get("transformed").and_then(|v| v.as_str()).unwrap_or("");
            Ok(HashMap::from([("output".into(), Value::String(format!("Processed: {}", text)))]))
        }),
    ));
    let chains: Vec<Arc<dyn Chain>> = vec![preprocess, postprocess];
    let seq = SimpleSequentialChain::new(chains);
    let result = seq.call(HashMap::from([("input".into(), Value::String("hello".into()))])).await?;
    println!("Sequential: {}", result.get("output").and_then(|v| v.as_str()).unwrap_or(""));
    Ok(())
}

async fn tools_demo() -> Result<()> {
    use langchain_tools::traits::BaseTool;
    use langchain_tools::utility::{CalculatorTool, ShellTool};

    let calc = CalculatorTool;
    let result = calc.invoke("2 + 3 * 4").await?;
    println!("Calculator: 2 + 3 * 4 = {}", result);

    let shell = ShellTool::default();
    match shell.invoke("echo 'Hello from ShellTool'").await {
        Ok(r) => println!("Shell: {}", r.trim()),
        Err(e) => println!("Shell error (expected on some systems): {}", e),
    }
    Ok(())
}

async fn document_loaders() -> Result<()> {
    use langchain_document_loaders::text::TextLoader;
    use langchain_document_loaders::traits::BaseLoader;

    let temp_path = std::env::temp_dir().join("langchain_test_doc.txt");
    tokio::fs::write(&temp_path, b"Hello from the test document!\nThis is line two.").await?;
    let loader = TextLoader::new(temp_path.to_str().unwrap());
    let docs = loader.load().await?;
    println!("TextLoader: {} docs loaded", docs.len());
    for doc in &docs {
        println!("  Content: {}", doc.page_content);
    }
    tokio::fs::remove_file(&temp_path).await?;
    Ok(())
}

fn chat_prompts() -> Result<()> {
    let template = ChatPromptTemplate::new()
        .add_system("You are a helpful assistant that knows about {subject}.")
        .add_human("Tell me about {topic}.");
    for msg in &template.messages {
        match msg {
            MessageTemplate::System(t) => println!("  [System] {}", t.replace("{subject}", "Rust")),
            MessageTemplate::Human(t) => println!("  [Human] {}", t.replace("{topic}", "ownership")),
            MessageTemplate::AI(t) => println!("  [AI] {}", t),
            MessageTemplate::Placeholder(_) => {}
        }
    }
    Ok(())
}

async fn conversational_retrieval_demo() -> Result<()> {
    use langchain_vectorstores::memory::InMemoryVectorStore;
    
    use langchain_embeddings::fake::FakeEmbeddings;

    let llm = Arc::new(FakeListLLM::new(vec!["Rust is a safe systems language.".into()]));
    let embeddings = Arc::new(FakeEmbeddings::new(4));
    let vectorstore = Arc::new(InMemoryVectorStore::new(embeddings));
    let chain = ConversationalRetrievalChain::new(llm, vectorstore);
    let result = chain.call(HashMap::from([
        ("question".into(), Value::String("What is Rust?".into())),
        ("chat_history".into(), Value::String("".into())),
    ])).await?;
    println!("Conversational QA: {}", result.get("answer").and_then(|v| v.as_str()).unwrap_or(""));
    Ok(())
}
