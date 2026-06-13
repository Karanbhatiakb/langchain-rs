use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use langchain_agents::executor::AgentExecutor;
use langchain_agents::react::ReActAgent;
use langchain_chains::llm_chain::LLMChain;
use langchain_core::callbacks::CallbackManager;
use langchain_core::callbacks::StdOutCallbackHandler;
use langchain_core::documents::Document;
use langchain_core::messages::HumanMessage;
use langchain_core::output_parsers::CommaSeparatedListOutputParser;
use langchain_core::output_parsers::OutputParser;
use langchain_core::output_parsers::StrOutputParser;
use langchain_core::prompt::PromptTemplate;
use langchain_core::runnable::Runnable;
use langchain_core::text_splitters::RecursiveCharacterTextSplitter;
use langchain_core::text_splitters::TextSplitter;
use langchain_document_loaders::traits::BaseLoader;
use langchain_llms::fake::FakeListLLM;
use langchain_llms::traits::ChatModel;
use langchain_memory::traits::BaseMemory;
use langchain_tools::traits::BaseTool;
use langchain_tools::utility::CalculatorTool;
use langchain_vectorstores::traits::VectorStore;
use serde_json::Value;

fn print_separator(title: &str) {
    println!("\n{}", "=".repeat(60));
    println!("  {}", title);
    println!("{}", "=".repeat(60));
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Creating an LLM (FakeLLM)
    print_separator("1. Creating an LLM (FakeListLLM)");
    let llm = Arc::new(FakeListLLM::new(vec![
        "Rust is a systems programming language focused on safety and performance.".into(),
        "The answer is 42.".into(),
        "I am an AI assistant with access to tools.".into(),
    ]));
    println!("FakeListLLM created with 3 canned responses");

    // 2. Creating a prompt template
    print_separator("2. Creating a prompt template");
    let prompt = PromptTemplate::from_template("Tell me about {topic} in the context of {area}");
    println!("Prompt template: '{}'", prompt.template);
    println!("Input variables: {:?}", prompt.input_variables);

    // 3. Creating an LLMChain
    print_separator("3. Creating an LLMChain");
    let chain: LLMChain<String> = LLMChain::new(llm.clone(), prompt.clone());
    let result = chain
        .invoke(HashMap::from([
            ("topic".into(), Value::String("Rust".into())),
            ("area".into(), Value::String("programming".into())),
        ]))
        .await?;
    println!("LLMChain result: {}", result);

    // 4. Creating tools
    print_separator("4. Creating tools");
    let calculator = CalculatorTool;
    let calc_result = calculator.invoke("2 + 3 * 4").await?;
    println!("Calculator tool: 2 + 3 * 4 = {}", calc_result);

    struct SearchStub;
    #[async_trait::async_trait]
    impl BaseTool for SearchStub {
        fn name(&self) -> &str {
            "search"
        }
        fn description(&self) -> &str {
            "Search the web for information. Input should be a search query."
        }
        async fn invoke(&self, input: &str) -> langchain_core::errors::Result<String> {
            Ok(format!("Result for '{}': Found some relevant information.", input))
        }
    }
    let search = SearchStub;
    println!("Search stub tool created");
    println!("Search result: {}", search.invoke("Rust programming").await?);

    // 5. Creating an agent with the tools
    print_separator("5. Creating an agent with tools");
    let tools: Vec<Arc<dyn BaseTool>> = vec![Arc::new(calculator), Arc::new(search)];
    let agent_llm = Arc::new(FakeListLLM::new(vec![
        "Thought: I need to calculate\nAction: calculator\nAction Input: 2+2\nObservation: 4\nFinal Answer: The answer is 4".into(),
    ]));
    let agent = ReActAgent::new(agent_llm.clone(), tools.clone(), None);
    let executor = AgentExecutor::new(Arc::new(agent), tools.clone());
    let agent_result = executor
        .run(&HashMap::from([("input".into(), Value::String("What is 2+2?".into()))]))
        .await?;
    println!("Agent result: {}", agent_result);

    // 6. Adding memory
    print_separator("6. Adding memory");
    let memory_llm = Arc::new(FakeListLLM::new(vec![
        "Hello! How can I help you?".into(),
        "I'm doing well, thanks!".into(),
        "I like programming in Rust.".into(),
    ]));
    let memory =
        Arc::new(langchain_memory::buffer::ConversationBufferMemory::new().with_return_messages(false));
    let mem_prompt =
        PromptTemplate::from_template("The following is a conversation:\n{history}\nHuman: {input}\nAI:");

    for input in &["Hi there!", "How are you?", "What do you like?"] {
        let vars = memory.load_memory_variables(&HashMap::new()).await?;
        let chat_history = vars.get("history").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut format_inputs = HashMap::new();
        format_inputs.insert("input".to_string(), input.to_string());
        format_inputs.insert("history".to_string(), chat_history);
        let formatted = mem_prompt.format(&format_inputs)?;
        let msg = memory_llm
            .predict_messages(&[HumanMessage::new(&formatted).into()], None, None)
            .await?;
        let output = msg.content;
        memory
            .save_context(
                &HashMap::from([("input".into(), Value::String(input.to_string()))]),
                &HashMap::from([("output".into(), Value::String(output.clone()))]),
            )
            .await?;
        println!("Human: {} | AI: {}", input, output);
    }
    println!("Memory buffer:\n{}", memory.buffer());

    // 7. Adding callbacks
    print_separator("7. Adding callbacks");
    let callback_llm = Arc::new(FakeListLLM::new(vec!["Hello from callback LLM".into()]));
    let cb_prompt = PromptTemplate::from_template("Say {word}");
    let mut cb_manager = CallbackManager::new();
    cb_manager.add_handler(Arc::new(StdOutCallbackHandler));
    let cb_chain: LLMChain<String> = LLMChain::new(callback_llm.clone(), cb_prompt).with_callbacks(cb_manager);
    let cb_result = cb_chain
        .invoke(HashMap::from([("word".into(), Value::String("hello".into()))]))
        .await?;
    println!("Callback chain result: {}", cb_result);

    // 8. Running the agent (already done above, but calling again to show reuse)
    print_separator("8. Running the agent (revisited)");
    let re_executor = AgentExecutor::new(
        Arc::new(ReActAgent::new(
            Arc::new(FakeListLLM::new(vec![
                "Thought: I have the answer\nFinal Answer: The answer is Rust!".into(),
            ])),
            tools.clone(),
            None,
        )),
        tools,
    );
    let re_result = re_executor
        .run(&HashMap::from([("input".into(), Value::String("What is Rust?".into()))]))
        .await?;
    println!("Agent result: {}", re_result);

    // 9. Using a vector store (InMemoryVectorStore)
    print_separator("9. Using a vector store (InMemoryVectorStore)");
    let embeddings = Arc::new(langchain_embeddings::fake::FakeEmbeddings::new(4));
    let store = langchain_vectorstores::memory::InMemoryVectorStore::new(embeddings);
    println!("InMemoryVectorStore created with FakeEmbeddings (dim=4)");

    // 10. Using embeddings (FakeEmbeddings)
    print_separator("10. Using embeddings (FakeEmbeddings)");
    let query_vec = store.embeddings().embed_query("Rust programming").await?;
    println!("FakeEmbeddings for 'Rust programming': {:?}", &query_vec);

    // 11. Adding documents to the vector store
    print_separator("11. Adding documents to the vector store");
    let doc_ids = store
        .add_documents(vec![
            Document::new("Rust is a systems programming language"),
            Document::new("Python is a high-level programming language"),
            Document::new("JavaScript runs in the browser"),
            Document::new("Rust focuses on safety and performance"),
            Document::new("C++ is a multi-paradigm systems language"),
        ])
        .await?;
    println!("Added {} documents, IDs: {:?}", doc_ids.len(), doc_ids);

    // 12. Searching the vector store
    print_separator("12. Searching the vector store");
    let search_results = store.similarity_search("Rust", 3).await?;
    println!("Similarity search for 'Rust' (top 3):");
    for doc in &search_results {
        println!("  - {}", doc.page_content);
    }

    let scored_results = store.similarity_search_with_score("programming", 2).await?;
    println!("Similarity search with score for 'programming':");
    for (doc, score) in &scored_results {
        println!("  [{:.4}] {}", score, doc.page_content);
    }

    // 13. Using document loaders (TextLoader)
    print_separator("13. Using document loaders (TextLoader)");
    let temp_path = std::env::temp_dir().join("langchain_complete_demo.txt");
    tokio::fs::write(
        &temp_path,
        b"This is a test document for the LangChain Rust demo.\nIt has multiple lines of text.\nWe will load it and process it.",
    )
    .await?;
    let loader = langchain_document_loaders::text::TextLoader::new(temp_path.to_str().unwrap());
    let docs = loader.load().await?;
    println!("TextLoader loaded {} document(s):", docs.len());
    for doc in &docs {
        println!("  Content: {}", doc.page_content);
    }
    tokio::fs::remove_file(&temp_path).await?;

    // 14. Using output parsers
    print_separator("14. Using output parsers");
    let str_parser = StrOutputParser;
    let parsed_str = str_parser.parse("hello world")?;
    println!("StrOutputParser: '{}'", parsed_str);

    let list_parser = CommaSeparatedListOutputParser;
    let parsed_list = list_parser.parse("apple, banana, cherry")?;
    println!("CommaSeparatedListOutputParser: {:?}", parsed_list);

    // 15. Using text splitters
    print_separator("15. Using text splitters");
    let text = "This is the first paragraph about Rust.\n\nThis is the second paragraph about safety.\n\nThis is the third paragraph about performance.\n\nThis is the fourth and final paragraph.";
    let splitter = RecursiveCharacterTextSplitter::new(
        vec!["\n\n".into(), "\n".into(), " ".into(), "".into()],
        60,
        10,
    );
    let chunks = splitter.split_text(text)?;
    println!(
        "RecursiveCharacterTextSplitter created {} chunks:",
        chunks.len()
    );
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: '{}'", i + 1, chunk);
    }

    // All done
    print_separator("Complete demo finished successfully");
    Ok(())
}
