//! Graph QA chain implementation — answers questions over knowledge graphs.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSchema {
    pub entity_types: Vec<String>,
    pub relationship_types: Vec<String>,
    pub entity_properties: HashMap<String, Vec<String>>,
    pub relationship_properties: HashMap<String, Vec<String>>,
}

#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn execute_query(&self, query: &str) -> Result<Vec<HashMap<String, Value>>>;
    fn schema(&self) -> &GraphSchema;
    fn query_language(&self) -> QueryLanguage;
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryLanguage {
    Cypher,
    SPARQL,
}

pub struct GraphQAChain {
    llm: Arc<dyn ChatModel>,
    graph_store: Option<Arc<dyn GraphStore>>,
    query_prompt: PromptTemplate,
    qa_prompt: PromptTemplate,
    verbose: bool,
}

impl GraphQAChain {
    pub fn new(llm: Arc<dyn ChatModel>) -> Self {
        let query_prompt = PromptTemplate::from_template(
            "You are a graph database query generator. Given a question and a graph schema, generate a query.\n\n\
            Graph Schema:\n\
            Entity types: {entity_types}\n\
            Relationship types: {relationship_types}\n\n\
            Question: {question}\n\n\
            Generate a {query_language} query to answer the question.\n\
            Respond with ONLY the query, no explanation.\n\n\
            Query:",
        );
        let qa_prompt = PromptTemplate::from_template(
            "You are a knowledge graph QA assistant. Given the question and the query results, provide a natural language answer.\n\n\
            Question: {question}\n\n\
            Query Results:\n{query_results}\n\n\
            Answer:",
        );

        Self {
            llm,
            graph_store: None,
            query_prompt,
            qa_prompt,
            verbose: false,
        }
    }

    pub fn with_graph_store(mut self, store: Arc<dyn GraphStore>) -> Self {
        self.graph_store = Some(store);
        self
    }

    pub fn with_query_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.query_prompt = prompt;
        self
    }

    pub fn with_qa_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.qa_prompt = prompt;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    fn extract_schema_info(&self, schema: &GraphSchema) -> (String, String) {
        let entity_types = schema.entity_types.join(", ");
        let relationship_types = schema.relationship_types.join(", ");
        (entity_types, relationship_types)
    }

    async fn generate_query(&self, question: &str, schema: &GraphSchema, query_language: &QueryLanguage) -> Result<String> {
        let (entity_types, relationship_types) = self.extract_schema_info(schema);
        let lang_str = match query_language {
            QueryLanguage::Cypher => "Cypher",
            QueryLanguage::SPARQL => "SPARQL",
        };

        let mut kwargs = HashMap::new();
        kwargs.insert("question".to_string(), question.to_string());
        kwargs.insert("entity_types".to_string(), entity_types);
        kwargs.insert("relationship_types".to_string(), relationship_types);
        kwargs.insert("query_language".to_string(), lang_str.to_string());
        let formatted = self.query_prompt.format(&kwargs)?;

        let messages = vec![HumanMessage::new(&formatted).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        let query = response.content.trim().to_string();
        let cleaned = self.clean_query(&query);
        Ok(cleaned)
    }

    fn clean_query(&self, query: &str) -> String {
        let cleaned = query.trim().to_string();
        let re = Regex::new(r"```(?:cypher|sparql|sql)?\s*").unwrap();
        let cleaned = re.replace_all(&cleaned, "").to_string();
        let re = Regex::new(r"```").unwrap();
        re.replace_all(&cleaned, "").trim().to_string()
    }

    async fn execute_query(&self, query: &str) -> Result<Vec<HashMap<String, Value>>> {
        if let Some(ref store) = self.graph_store {
            store.execute_query(query).await
        } else {
            Ok(Vec::new())
        }
    }

    async fn generate_answer(
        &self,
        question: &str,
        query_results: &[HashMap<String, Value>],
    ) -> Result<String> {
        let results_str = serde_json::to_string_pretty(query_results)
            .map_err(|e| ChainError::SerializationError(e.to_string()))?;

        let mut kwargs = HashMap::new();
        kwargs.insert("question".to_string(), question.to_string());
        kwargs.insert("query_results".to_string(), results_str);
        let formatted = self.qa_prompt.format(&kwargs)?;

        let messages = vec![HumanMessage::new(&formatted).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;
        Ok(response.content)
    }
}

#[async_trait]
impl Chain for GraphQAChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["question".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string(), "query".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.verbose {
            info!("GraphQAChain processing: {}", question);
        }

        if let Some(ref store) = self.graph_store {
            let schema = store.schema();
            let query_language = store.query_language();

            let query = self.generate_query(&question, schema, &query_language).await?;

            if self.verbose {
                info!("GraphQAChain generated query: {}", query);
            }

            let query_results = self.execute_query(&query).await?;

            if self.verbose {
                info!("GraphQAChain got {} result rows", query_results.len());
            }

            let answer = self.generate_answer(&question, &query_results).await?;

            let mut result = HashMap::new();
            result.insert("output".to_string(), Value::String(answer));
            result.insert("query".to_string(), Value::String(query));
            Ok(result)
        } else {
            let answer = self.generate_answer(&question, &[]).await?;
            let mut result = HashMap::new();
            result.insert("output".to_string(), Value::String(answer));
            result.insert("query".to_string(), Value::String(String::new()));
            Ok(result)
        }
    }
}
