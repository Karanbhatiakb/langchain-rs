//! Document compressors for retrieval.
//!
//! Provides the [`DocumentCompressor`] trait and implementations:
//! - [`NoopDocumentCompressor`] — passes documents through unchanged
//! - [`ScoreThresholdCompressor`] — filters by minimum relevance score
//! - [`DocumentCompressorPipeline`] — chains multiple compressors together
//! - [`LLMChainExtractor`] — extracts relevant snippets via an LLM
//! - [`LLMChainFilter`] — filters documents via an LLM
//! - [`EmbeddingsFilter`] — filters by embedding similarity threshold
//! - [`CrossEncoderReranker`] — reranks using a cross-encoder model

use crate::cross_encoders::CrossEncoder;
use crate::documents::Document;
use crate::embeddings::Embeddings;
use crate::errors::*;
use crate::language_models::BaseChatModel;
use crate::prompt::ChatPromptTemplate;
use crate::language_models::PromptValue;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for compressing or filtering retrieved documents.
#[async_trait]
pub trait DocumentCompressor: Send + Sync + 'static {
    /// Compresses the given documents relative to the query, returning a
    /// potentially smaller or re-ranked list of documents.
    async fn compress_documents(
        &self,
        documents: Vec<Document>,
        query: &str,
    ) -> Result<Vec<Document>>;

    /// Alias for [`compress_documents`].
    async fn compress(
        &self,
        documents: Vec<Document>,
        query: &str,
    ) -> Result<Vec<Document>> {
        self.compress_documents(documents, query).await
    }
}

/// A no-op compressor that returns documents unchanged.
#[derive(Debug, Clone)]
pub struct NoopDocumentCompressor;

impl NoopDocumentCompressor {
    /// Creates a new `NoopDocumentCompressor`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopDocumentCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentCompressor for NoopDocumentCompressor {
    async fn compress_documents(
        &self,
        documents: Vec<Document>,
        _query: &str,
    ) -> Result<Vec<Document>> {
        Ok(documents)
    }
}

/// A compressor that filters documents by a minimum relevance score.
#[derive(Debug, Clone)]
pub struct ScoreThresholdCompressor {
    /// The minimum score a document must have to be retained.
    pub threshold: f32,
}

impl ScoreThresholdCompressor {
    /// Creates a new `ScoreThresholdCompressor` with the given threshold.
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

#[async_trait]
impl DocumentCompressor for ScoreThresholdCompressor {
    async fn compress_documents(
        &self,
        documents: Vec<Document>,
        _query: &str,
    ) -> Result<Vec<Document>> {
        Ok(documents
            .into_iter()
            .filter(|doc| doc.score.unwrap_or(0.0) >= self.threshold)
            .collect())
    }
}

/// A pipeline that chains multiple [`DocumentCompressor`]s together.
///
/// Compressors are applied in order — the output of one becomes the input of
/// the next.
pub struct DocumentCompressorPipeline {
    compressors: Vec<Box<dyn DocumentCompressor>>,
}

impl std::fmt::Debug for DocumentCompressorPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentCompressorPipeline")
            .field("num_compressors", &self.compressors.len())
            .finish()
    }
}

impl DocumentCompressorPipeline {
    /// Creates an empty pipeline.
    pub fn new() -> Self {
        Self {
            compressors: Vec::new(),
        }
    }

    /// Appends a compressor to the pipeline (builder pattern).
    pub fn add_compressor(mut self, compressor: impl DocumentCompressor) -> Self {
        self.compressors.push(Box::new(compressor));
        self
    }

    /// Returns a slice of the compressors in the pipeline.
    pub fn compressors(&self) -> &[Box<dyn DocumentCompressor>] {
        &self.compressors
    }
}

impl Default for DocumentCompressorPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentCompressor for DocumentCompressorPipeline {
    async fn compress_documents(
        &self,
        documents: Vec<Document>,
        query: &str,
    ) -> Result<Vec<Document>> {
        let mut docs = documents;
        for compressor in &self.compressors {
            docs = compressor.compress_documents(docs, query).await?;
        }
        Ok(docs)
    }
}

/// A compressor that uses an LLM to extract relevant snippets from each
/// document.
///
/// For each input document, the LLM is prompted with the query and document
/// content. Documents for which the LLM returns empty text are dropped.
pub struct LLMChainExtractor<M: BaseChatModel> {
    llm: Arc<M>,
    prompt: ChatPromptTemplate,
    max_concurrency: usize,
}

impl<M: BaseChatModel> std::fmt::Debug for LLMChainExtractor<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMChainExtractor")
            .field("prompt", &self.prompt)
            .field("max_concurrency", &self.max_concurrency)
            .finish()
    }
}

impl<M: BaseChatModel> Clone for LLMChainExtractor<M> {
    fn clone(&self) -> Self {
        Self {
            llm: self.llm.clone(),
            prompt: self.prompt.clone(),
            max_concurrency: self.max_concurrency,
        }
    }
}

/// Default prompt template for [`LLMChainExtractor`].
fn default_extractor_prompt() -> ChatPromptTemplate {
    ChatPromptTemplate::new()
        .add_system("You are an expert at extracting relevant information from documents.")
        .add_human("Given the question: {query}\n\nExtract from the following document any text relevant to answering the question:\n\n{document}")
}

impl<M: BaseChatModel> LLMChainExtractor<M> {
    /// Creates a new `LLMChainExtractor` wrapping the given chat model.
    ///
    /// Uses a default extraction prompt.
    pub fn new(llm: M) -> Self {
        Self {
            llm: Arc::new(llm),
            prompt: default_extractor_prompt(),
            max_concurrency: 5,
        }
    }

    /// Replaces the prompt template (builder pattern).
    ///
    /// The template should accept `{query}` and `{document}` variables.
    pub fn with_prompt(mut self, prompt: ChatPromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    /// Sets the maximum number of concurrent LLM calls (builder pattern).
    pub fn with_max_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = n;
        self
    }
}

#[async_trait]
impl<M: BaseChatModel> DocumentCompressor for LLMChainExtractor<M> {
    async fn compress_documents(
        &self,
        documents: Vec<Document>,
        query: &str,
    ) -> Result<Vec<Document>> {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrency));
        let mut tasks = Vec::new();

        for doc in documents {
            let llm = self.llm.clone();
            let prompt = self.prompt.clone();
            let query_string = query.to_string();
            let sem = semaphore.clone();

            tasks.push(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| ChainError::LLMError(e.to_string()))?;

                let mut kwargs = HashMap::new();
                kwargs.insert("query".to_string(), query_string);
                kwargs.insert("document".to_string(), doc.page_content.clone());
                let messages = prompt.format_prompt(&kwargs)?;
                let pv = PromptValue::Messages(messages);
                let llm_result = llm.generate_prompt(vec![pv], None).await?;
                let text = llm_result
                    .generations
                    .into_iter()
                    .next()
                    .and_then(|mut gens| gens.pop())
                    .map(|g| g.text)
                    .unwrap_or_default();

                if text.trim().is_empty() {
                    return Ok::<Option<Document>, crate::errors::ChainError>(None);
                }
                let mut new_doc = doc;
                new_doc.page_content = text;
                Ok(Some(new_doc))
            });
        }

        let results: Vec<Option<Document>> =
            futures::future::try_join_all(tasks).await?;
        Ok(results.into_iter().flatten().collect())
    }
}

/// A compressor that uses an LLM to decide whether each document is relevant
/// to the query.
///
/// The LLM is prompted to answer 'YES' or 'NO'. Documents classified as
/// relevant are retained; the rest are dropped.
pub struct LLMChainFilter<M: BaseChatModel> {
    llm: Arc<M>,
    prompt: ChatPromptTemplate,
    max_concurrency: usize,
}

impl<M: BaseChatModel> std::fmt::Debug for LLMChainFilter<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMChainFilter")
            .field("prompt", &self.prompt)
            .field("max_concurrency", &self.max_concurrency)
            .finish()
    }
}

impl<M: BaseChatModel> Clone for LLMChainFilter<M> {
    fn clone(&self) -> Self {
        Self {
            llm: self.llm.clone(),
            prompt: self.prompt.clone(),
            max_concurrency: self.max_concurrency,
        }
    }
}

/// Default prompt template for [`LLMChainFilter`].
fn default_filter_prompt() -> ChatPromptTemplate {
    ChatPromptTemplate::new()
        .add_system("You are an expert at determining document relevance.")
        .add_human(
            "Determine if the following document is relevant to the question: {query}\n\n\
             Document: {document}\n\n\
             Answer with exactly 'YES' or 'NO':",
        )
}

impl<M: BaseChatModel> LLMChainFilter<M> {
    /// Creates a new `LLMChainFilter` wrapping the given chat model.
    ///
    /// Uses a default relevance prompt.
    pub fn new(llm: M) -> Self {
        Self {
            llm: Arc::new(llm),
            prompt: default_filter_prompt(),
            max_concurrency: 5,
        }
    }

    /// Replaces the prompt template (builder pattern).
    ///
    /// The template should accept `{query}` and `{document}` variables and
    /// elicit a 'YES' or 'NO' answer.
    pub fn with_prompt(mut self, prompt: ChatPromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    /// Sets the maximum number of concurrent LLM calls (builder pattern).
    pub fn with_max_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = n;
        self
    }
}

#[async_trait]
impl<M: BaseChatModel> DocumentCompressor for LLMChainFilter<M> {
    async fn compress_documents(
        &self,
        documents: Vec<Document>,
        query: &str,
    ) -> Result<Vec<Document>> {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrency));
        let mut tasks = Vec::new();

        for doc in documents {
            let llm = self.llm.clone();
            let prompt = self.prompt.clone();
            let query_string = query.to_string();
            let sem = semaphore.clone();

            tasks.push(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| ChainError::LLMError(e.to_string()))?;

                let mut kwargs = HashMap::new();
                kwargs.insert("query".to_string(), query_string);
                kwargs.insert("document".to_string(), doc.page_content.clone());
                let messages = prompt.format_prompt(&kwargs)?;
                let pv = PromptValue::Messages(messages);
                let llm_result = llm.generate_prompt(vec![pv], None).await?;
                let text = llm_result
                    .generations
                    .into_iter()
                    .next()
                    .and_then(|mut gens| gens.pop())
                    .map(|g| g.text)
                    .unwrap_or_default();

                let is_relevant = text.trim().to_uppercase().contains("YES");
                if is_relevant {
                    Ok::<Option<Document>, crate::errors::ChainError>(Some(doc))
                } else {
                    Ok::<Option<Document>, crate::errors::ChainError>(None)
                }
            });
        }

        let results: Vec<Option<Document>> =
            futures::future::try_join_all(tasks).await?;
        Ok(results.into_iter().flatten().collect())
    }
}

/// A compressor that filters documents by cosine similarity between their
/// embeddings and the query embedding.
///
/// Documents whose similarity to the query is below the configured threshold
/// are dropped. An optional `top_k` limit can be set to keep only the top-K
/// most similar documents.
pub struct EmbeddingsFilter<E: Embeddings> {
    embeddings: Arc<E>,
    /// Minimum cosine similarity to retain a document.
    pub similarity_threshold: f64,
    /// Optional maximum number of documents to keep.
    pub k: Option<usize>,
}

impl<E: Embeddings> std::fmt::Debug for EmbeddingsFilter<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingsFilter")
            .field("similarity_threshold", &self.similarity_threshold)
            .field("k", &self.k)
            .finish()
    }
}

impl<E: Embeddings> Clone for EmbeddingsFilter<E> {
    fn clone(&self) -> Self {
        Self {
            embeddings: self.embeddings.clone(),
            similarity_threshold: self.similarity_threshold,
            k: self.k,
        }
    }
}

impl<E: Embeddings> EmbeddingsFilter<E> {
    /// Creates a new `EmbeddingsFilter`.
    ///
    /// # Arguments
    /// * `embeddings` — the embedding model.
    /// * `similarity_threshold` — documents below this cosine similarity are
    ///   dropped.
    pub fn new(embeddings: E, similarity_threshold: f64) -> Self {
        Self {
            embeddings: Arc::new(embeddings),
            similarity_threshold,
            k: None,
        }
    }

    /// Sets the `k` limit (builder pattern).
    ///
    /// When set, only the top-K documents by similarity are retained.
    pub fn with_k(mut self, k: usize) -> Self {
        self.k = Some(k);
        self
    }
}

/// Computes cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum();
    let norm_b: f64 = b.iter().map(|x| x * x).sum();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}

#[async_trait]
impl<E: Embeddings> DocumentCompressor for EmbeddingsFilter<E> {
    async fn compress_documents(
        &self,
        documents: Vec<Document>,
        query: &str,
    ) -> Result<Vec<Document>> {
        let query_embedding = self.embeddings.embed_query(query).await?;
        let texts: Vec<String> = documents.iter().map(|d| d.page_content.clone()).collect();
        let doc_embeddings = self.embeddings.embed_documents(&texts).await?;

        let mut scored: Vec<(Document, f64)> = documents
            .into_iter()
            .zip(doc_embeddings.into_iter())
            .map(|(doc, emb)| {
                let sim = cosine_similarity(&query_embedding, &emb);
                (doc, sim)
            })
            .filter(|(_, sim)| *sim >= self.similarity_threshold)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(k) = self.k {
            scored.truncate(k);
        }

        Ok(scored
            .into_iter()
            .map(|(mut doc, sim)| {
                doc.score = Some(sim as f32);
                doc
            })
            .collect())
    }
}

/// A compressor that reranks documents using a cross-encoder model.
///
/// All documents are scored against the query using the cross-encoder, then
/// the top-K highest-scoring documents are returned in descending score order.
pub struct CrossEncoderReranker<C: CrossEncoder + 'static> {
    cross_encoder: Arc<C>,
    /// The number of top documents to keep after reranking.
    pub top_k: usize,
}

impl<C: CrossEncoder + 'static> std::fmt::Debug for CrossEncoderReranker<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrossEncoderReranker")
            .field("top_k", &self.top_k)
            .finish()
    }
}

impl<C: CrossEncoder + 'static> Clone for CrossEncoderReranker<C> {
    fn clone(&self) -> Self {
        Self {
            cross_encoder: self.cross_encoder.clone(),
            top_k: self.top_k,
        }
    }
}

impl<C: CrossEncoder + 'static> CrossEncoderReranker<C> {
    /// Creates a new `CrossEncoderReranker`.
    ///
    /// # Arguments
    /// * `cross_encoder` — the cross-encoder model.
    /// * `top_k` — number of top documents to retain.
    pub fn new(cross_encoder: C, top_k: usize) -> Self {
        Self {
            cross_encoder: Arc::new(cross_encoder),
            top_k,
        }
    }
}

#[async_trait]
impl<C: CrossEncoder + 'static> DocumentCompressor for CrossEncoderReranker<C> {
    async fn compress_documents(
        &self,
        documents: Vec<Document>,
        query: &str,
    ) -> Result<Vec<Document>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        let pairs: Vec<(String, String)> = documents
            .iter()
            .map(|d| (query.to_string(), d.page_content.clone()))
            .collect();

        let scores = self.cross_encoder.score(pairs).await?;

        let mut scored: Vec<(Document, f32)> = documents
            .into_iter()
            .zip(scores.into_iter())
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(self.top_k);

        Ok(scored
            .into_iter()
            .map(|(mut doc, score)| {
                doc.score = Some(score);
                doc
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_encoders::FakeCrossEncoder;
    use crate::embeddings::FakeEmbeddings;
    use crate::language_models::FakeListChatModel;

    #[tokio::test]
    async fn test_noop_compressor() {
        let compressor = NoopDocumentCompressor::new();
        let docs = vec![
            Document::new("doc1"),
            Document::new("doc2"),
            Document::new("doc3"),
        ];
        let result = compressor.compress_documents(docs, "query").await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].page_content, "doc1");
        assert_eq!(result[1].page_content, "doc2");
        assert_eq!(result[2].page_content, "doc3");
    }

    #[tokio::test]
    async fn test_score_threshold_compressor() {
        let compressor = ScoreThresholdCompressor::new(0.5);
        let docs = vec![
            Document::new("high").with_score(0.9),
            Document::new("medium").with_score(0.5),
            Document::new("low").with_score(0.1),
            Document::new("no_score"),
        ];
        let result = compressor.compress_documents(docs, "query").await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].page_content, "high");
        assert_eq!(result[1].page_content, "medium");
    }

    #[tokio::test]
    async fn test_score_threshold_compressor_all_pass() {
        let compressor = ScoreThresholdCompressor::new(0.0);
        let docs = vec![
            Document::new("a").with_score(0.5),
            Document::new("b").with_score(0.0),
        ];
        let result = compressor.compress_documents(docs, "query").await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_score_threshold_compressor_none_pass() {
        let compressor = ScoreThresholdCompressor::new(0.99);
        let docs = vec![
            Document::new("a").with_score(0.5),
            Document::new("b"),
        ];
        let result = compressor.compress_documents(docs, "query").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_empty() {
        let pipeline = DocumentCompressorPipeline::new();
        let docs = vec![Document::new("hello")];
        let result = pipeline
            .compress_documents(docs, "query")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_pipeline_with_noop() {
        let pipeline = DocumentCompressorPipeline::new()
            .add_compressor(NoopDocumentCompressor::new());
        let docs = vec![Document::new("hello")];
        let result = pipeline
            .compress_documents(docs, "query")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_pipeline_chained() {
        let pipeline = DocumentCompressorPipeline::new()
            .add_compressor(ScoreThresholdCompressor::new(0.5))
            .add_compressor(ScoreThresholdCompressor::new(0.8));
        let docs = vec![
            Document::new("a").with_score(0.9),
            Document::new("b").with_score(0.7),
            Document::new("c").with_score(0.3),
        ];
        let result = pipeline
            .compress_documents(docs, "query")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].page_content, "a");
    }

    #[tokio::test]
    async fn test_compress_alias() {
        let compressor = NoopDocumentCompressor::new();
        let docs = vec![Document::new("a")];
        let result = compressor.compress(docs, "q").await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_cross_encoder_reranker() {
        let encoder = FakeCrossEncoder::new();
        let reranker = CrossEncoderReranker::new(encoder, 2);
        let docs = vec![
            Document::new("short"),
            Document::new("a much longer string here"),
            Document::new("tiny"),
        ];
        let result = reranker
            .compress_documents(docs, "query")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].score.is_some());
        assert!(result[1].score.is_some());
    }

    #[tokio::test]
    async fn test_cross_encoder_reranker_empty() {
        let encoder = FakeCrossEncoder::new();
        let reranker = CrossEncoderReranker::new(encoder, 5);
        let result = reranker
            .compress_documents(vec![], "query")
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_cross_encoder_reranker_top_k_larger_than_input() {
        let encoder = FakeCrossEncoder::new();
        let reranker = CrossEncoderReranker::new(encoder, 100);
        let docs = vec![Document::new("hello"), Document::new("world")];
        let result = reranker
            .compress_documents(docs, "query")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_embeddings_filter_all_above_threshold() {
        let embeddings = FakeEmbeddings::new(8);
        let filter = EmbeddingsFilter::new(embeddings, 0.0).with_k(5);
        let docs = vec![Document::new("hello"), Document::new("world")];
        let result = filter
            .compress_documents(docs, "test_query")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].score.is_some());
    }

    #[tokio::test]
    async fn test_embeddings_filter_top_k() {
        let embeddings = FakeEmbeddings::new(8);
        let filter = EmbeddingsFilter::new(embeddings, 0.0).with_k(1);
        let docs = vec![
            Document::new("alpha"),
            Document::new("beta"),
            Document::new("gamma"),
        ];
        let result = filter
            .compress_documents(docs, "test_query")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_embeddings_filter_empty_docs() {
        let embeddings = FakeEmbeddings::new(8);
        let filter = EmbeddingsFilter::new(embeddings, 0.5);
        let result = filter
            .compress_documents(vec![], "query")
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_llm_chain_extractor_with_fake() {
        let llm = FakeListChatModel::new(vec!["extracted snippet".to_string()]);
        let extractor = LLMChainExtractor::new(llm);
        let docs = vec![Document::new("some document content here")];
        let result = extractor
            .compress_documents(docs, "what is this about?")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result[0].page_content.is_empty());
        assert_eq!(result[0].page_content, "extracted snippet");
    }

    #[tokio::test]
    async fn test_llm_chain_filter_with_fake() {
        let llm = FakeListChatModel::new(vec!["YES".to_string()]);
        let filter = LLMChainFilter::new(llm);
        let docs = vec![Document::new("relevant content")];
        let result = filter
            .compress_documents(docs, "test query")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_llm_chain_filter_rejects_no() {
        let llm = FakeListChatModel::new(vec!["NO".to_string()]);
        let filter = LLMChainFilter::new(llm);
        let docs = vec![Document::new("irrelevant content")];
        let result = filter
            .compress_documents(docs, "test query")
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_llm_chain_extractor_custom_prompt() {
        let llm = FakeListChatModel::new(vec!["extracted data".to_string()]);
        let prompt = ChatPromptTemplate::new()
            .add_human("Q: {query}\nD: {document}\nExtract:");
        let extractor = LLMChainExtractor::new(llm).with_prompt(prompt);
        let docs = vec![Document::new("hello world")];
        let result = extractor
            .compress_documents(docs, "test")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_llm_chain_filter_custom_prompt() {
        let llm = FakeListChatModel::new(vec!["YES".to_string()]);
        let prompt = ChatPromptTemplate::new()
            .add_human("Relevant? {query}\n{document}\nAnswer YES or NO:");
        let filter = LLMChainFilter::new(llm).with_prompt(prompt);
        let docs = vec![Document::new("hello world")];
        let result = filter
            .compress_documents(docs, "test")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 2.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_llm_chain_extractor_drops_empty() {
        let llm = FakeListChatModel::new(vec!["   ".to_string()]);
        let extractor = LLMChainExtractor::new(llm);
        let docs = vec![Document::new("something")];
        let result = extractor
            .compress_documents(docs, "query")
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_document_compressor_pipeline_debug() {
        let pipeline = DocumentCompressorPipeline::new();
        let debug = format!("{:?}", pipeline);
        assert!(debug.contains("num_compressors"));
    }

    #[tokio::test]
    async fn test_document_compressor_pipeline_compressors_len() {
        let pipeline = DocumentCompressorPipeline::new()
            .add_compressor(NoopDocumentCompressor::new())
            .add_compressor(NoopDocumentCompressor::new());
        assert_eq!(pipeline.compressors().len(), 2);
    }

    #[tokio::test]
    async fn test_noop_compressor_default() {
        let compressor = NoopDocumentCompressor::default();
        let docs = vec![Document::new("test")];
        let result = compressor.compress_documents(docs, "q").await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_embeddings_filter_no_k() {
        let embeddings = FakeEmbeddings::new(4);
        let filter = EmbeddingsFilter::new(embeddings, 0.0);
        let docs = vec![Document::new("hello"), Document::new("world")];
        let result = filter.compress_documents(docs, "test").await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_embeddings_filter_high_threshold() {
        let embeddings = FakeEmbeddings::new(4);
        let filter = EmbeddingsFilter::new(embeddings, 1.1);
        let docs = vec![Document::new("hello")];
        let result = filter.compress_documents(docs, "test").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_cross_encoder_reranker_debug() {
        let encoder = FakeCrossEncoder::new();
        let reranker = CrossEncoderReranker::new(encoder, 3);
        let debug = format!("{:?}", reranker);
        assert!(debug.contains("top_k"));
    }

    #[tokio::test]
    async fn test_embeddings_filter_debug() {
        let embeddings = FakeEmbeddings::new(4);
        let filter = EmbeddingsFilter::new(embeddings, 0.5).with_k(10);
        let debug = format!("{:?}", filter);
        assert!(debug.contains("similarity_threshold"));
    }

    #[tokio::test]
    async fn test_llm_chain_extractor_max_concurrency() {
        let llm = FakeListChatModel::new(vec!["output".to_string()]);
        let extractor = LLMChainExtractor::new(llm).with_max_concurrency(3);
        let docs = vec![Document::new("doc1"), Document::new("doc2")];
        let result = extractor.compress_documents(docs, "q").await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_llm_chain_filter_max_concurrency() {
        let llm = FakeListChatModel::new(vec!["YES".to_string()]);
        let filter = LLMChainFilter::new(llm).with_max_concurrency(3);
        let docs = vec![Document::new("doc1")];
        let result = filter.compress_documents(docs, "q").await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_document_compressor_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<NoopDocumentCompressor>();
        assert_sync::<NoopDocumentCompressor>();
        assert_send::<ScoreThresholdCompressor>();
        assert_sync::<ScoreThresholdCompressor>();
        assert_send::<DocumentCompressorPipeline>();
        assert_sync::<DocumentCompressorPipeline>();
    }
}
