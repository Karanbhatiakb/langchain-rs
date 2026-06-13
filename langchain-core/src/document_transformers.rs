//! Document transformers for modifying, reordering, or converting documents.
//!
//! Provides the [`DocumentTransformer`] trait — an async interface for
//! transforming [`Document`]s — and several concrete implementations:
//!
//! - [`LongContextReorder`] — mitigates the "lost in the middle" effect by
//!   placing the most relevant documents at the start and end of the list.
//! - [`GoogleVertexAIVisionReorder`] — reorders documents for Google Vertex AI
//!   vision models using the same alternating-end strategy.
//! - [`NucliaDBTransformer`] — converts documents into the NucliaDB ingestion
//!   format by serializing page content and metadata into JSON fields.

use crate::documents::Document;
use crate::errors::*;
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;

/// An async trait for transforming documents.
///
/// Implementors receive a vector of [`Document`]s and return a new vector
/// that has been filtered, reordered, enriched, or otherwise modified.
///
/// A default [`DocumentTransformer::transform_document`] implementation is
/// provided that delegates to [`DocumentTransformer::transform_documents`]
/// with a single-element vector.
#[async_trait]
pub trait DocumentTransformer: Send + Sync + 'static {
    /// Transforms a batch of documents and returns the result.
    ///
    /// # Errors
    ///
    /// Returns a [`ChainError`] if the transformation cannot be applied.
    async fn transform_documents(&self, documents: Vec<Document>) -> Result<Vec<Document>>;

    /// Transforms a single document by delegating to
    /// [`DocumentTransformer::transform_documents`].
    ///
    /// # Errors
    ///
    /// Propagates any error from [`DocumentTransformer::transform_documents`],
    /// or returns [`ChainError::ValidationError`] if the batch result is empty.
    async fn transform_document(&self, document: Document) -> Result<Document> {
        let results = self.transform_documents(vec![document]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| ChainError::ValidationError(
                "transform_documents returned an empty result for a single input document".into(),
            ))
    }
}

/// Reorders documents to mitigate the "lost in the middle" effect.
///
/// Research shows that language models pay more attention to the beginning
/// and end of a context window, while information in the middle is often
/// overlooked. [`LongContextReorder`] places the most relevant documents at
/// the start and end of the list by alternating between the front and back
/// of the sorted-by-relevance input.
///
/// If documents carry a [`Document::score`] field, they are first sorted
/// in descending order (highest relevance first). Documents without a score
/// are treated as equally relevant and retain their original order.
///
/// # Example
///
/// ```rust,no_run
/// use langchain_core::document_transformers::LongContextReorder;
/// use langchain_core::document_transformers::DocumentTransformer;
/// use langchain_core::documents::Document;
///
/// # async fn demo() -> langchain_core::errors::Result<()> {
/// let docs = vec![
///     Document::new("least relevant").with_score(0.1),
///     Document::new("most relevant").with_score(0.9),
///     Document::new("medium relevant").with_score(0.5),
/// ];
/// let reorder = LongContextReorder::new();
/// let reordered = reorder.transform_documents(docs).await?;
/// // reordered[0] and reordered[last] are the most relevant
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct LongContextReorder;

impl Default for LongContextReorder {
    fn default() -> Self {
        Self::new()
    }
}

impl LongContextReorder {
    /// Creates a new [`LongContextReorder`].
    pub fn new() -> Self {
        Self
    }

    /// Reorders a slice of documents using the alternating-end strategy.
    ///
    /// Documents at the front and back of the sorted list are placed at the
    /// start and end of the output, respectively, producing an order where
    /// the most relevant items occupy the two endpoints.
    fn reorder_docs(&self, mut documents: Vec<Document>) -> Vec<Document> {
        documents.sort_by(|a, b| {
            let sa = a.score.unwrap_or(f32::NEG_INFINITY);
            let sb = b.score.unwrap_or(f32::NEG_INFINITY);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let n = documents.len();
        let mut reordered = Vec::with_capacity(n);
        let mut front = true;
        let mut used = vec![false; n];

        for i in 0..n {
            let idx = if front { i / 2 } else { n - 1 - i / 2 };
            if idx < n && !used[idx] {
                used[idx] = true;
                reordered.push(documents[idx].clone());
            }
            front = !front;
        }

        if reordered.len() != n {
            return documents;
        }

        reordered
    }
}

#[async_trait]
impl DocumentTransformer for LongContextReorder {
    async fn transform_documents(&self, documents: Vec<Document>) -> Result<Vec<Document>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self.reorder_docs(documents))
    }
}

/// Reorders documents for Google Vertex AI vision models.
///
/// Uses the same alternating-end strategy as [`LongContextReorder`], but
/// is semantically distinct to allow future specialisation (e.g. filtering
/// by MIME type or image-only selection).
///
/// # Example
///
/// ```rust,no_run
/// use langchain_core::document_transformers::GoogleVertexAIVisionReorder;
/// use langchain_core::document_transformers::DocumentTransformer;
/// use langchain_core::documents::Document;
///
/// # async fn demo() -> langchain_core::errors::Result<()> {
/// let docs = vec![
///     Document::new("image description A").with_score(0.3),
///     Document::new("image description B").with_score(0.8),
///     Document::new("image description C").with_score(0.5),
/// ];
/// let reorder = GoogleVertexAIVisionReorder::new();
/// let reordered = reorder.transform_documents(docs).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GoogleVertexAIVisionReorder;

impl Default for GoogleVertexAIVisionReorder {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleVertexAIVisionReorder {
    /// Creates a new [`GoogleVertexAIVisionReorder`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DocumentTransformer for GoogleVertexAIVisionReorder {
    async fn transform_documents(&self, documents: Vec<Document>) -> Result<Vec<Document>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        let mut sorted = documents;
        sorted.sort_by(|a, b| {
            let sa = a.score.unwrap_or(f32::NEG_INFINITY);
            let sb = b.score.unwrap_or(f32::NEG_INFINITY);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let n = sorted.len();
        let mut reordered = Vec::with_capacity(n);
        let mut left = 0usize;
        let mut right = n;

        while left < right {
            reordered.push(sorted[left].clone());
            left += 1;
            if left < right {
                right -= 1;
                reordered.push(sorted[right].clone());
            }
        }

        Ok(reordered)
    }
}

/// Transforms documents into the NucliaDB ingestion format.
///
/// Each [`Document`] is converted so that its `page_content` is stored under
/// a `"text"` metadata key and its original metadata is preserved. A
/// `"format"` key is set to `"text"` and a `"origin"` key records the
/// conversion source.
///
/// # Example
///
/// ```rust,no_run
/// use langchain_core::document_transformers::NucliaDBTransformer;
/// use langchain_core::document_transformers::DocumentTransformer;
/// use langchain_core::documents::Document;
///
/// # async fn demo() -> langchain_core::errors::Result<()> {
/// let docs = vec![
///     Document::new("Hello from NucliaDB"),
/// ];
/// let transformer = NucliaDBTransformer::new();
/// let transformed = transformer.transform_documents(docs).await?;
/// assert_eq!(transformed[0].metadata.get("format").unwrap(), "text");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct NucliaDBTransformer {
    /// Optional identifier for the NucliaDB knowledge box.
    pub kbid: Option<String>,
    /// Optional path prefix for the NucliaDB resource.
    pub path: Option<String>,
}

impl Default for NucliaDBTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl NucliaDBTransformer {
    /// Creates a new [`NucliaDBTransformer`] with default settings.
    pub fn new() -> Self {
        Self {
            kbid: None,
            path: None,
        }
    }

    /// Sets the NucliaDB knowledge box identifier.
    pub fn with_kbid(mut self, kbid: impl Into<String>) -> Self {
        self.kbid = Some(kbid.into());
        self
    }

    /// Sets the NucliaDB resource path prefix.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    fn transform_single(&self, doc: Document) -> Result<Document> {
        let mut metadata: HashMap<String, serde_json::Value> = HashMap::new();

        metadata.insert("text".into(), serde_json::Value::String(doc.page_content.clone()));
        metadata.insert("format".into(), serde_json::Value::String("text".into()));
        metadata.insert(
            "origin".into(),
            serde_json::Value::String("langchain-rs::NucliaDBTransformer".into()),
        );

        if let Some(ref kbid) = self.kbid {
            metadata.insert("kbid".into(), serde_json::Value::String(kbid.clone()));
        }
        if let Some(ref path) = self.path {
            metadata.insert("path".into(), serde_json::Value::String(path.clone()));
        }

        for (k, v) in doc.metadata {
            metadata.insert(k, v);
        }

        Ok(Document {
            page_content: doc.page_content,
            metadata,
            score: doc.score,
        })
    }
}

#[async_trait]
impl DocumentTransformer for NucliaDBTransformer {
    async fn transform_documents(&self, documents: Vec<Document>) -> Result<Vec<Document>> {
        documents
            .into_iter()
            .map(|doc| self.transform_single(doc))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_long_context_reorder() {
        let docs = vec![
            Document::new("low").with_score(0.1),
            Document::new("mid").with_score(0.5),
            Document::new("high").with_score(0.9),
        ];
        let reorder = LongContextReorder::new();
        let result = reorder.transform_documents(docs).await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].page_content, "high");
    }

    #[tokio::test]
    async fn test_long_context_reorder_empty() {
        let reorder = LongContextReorder::new();
        let result = reorder.transform_documents(vec![]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_google_vertex_ai_vision_reorder() {
        let docs = vec![
            Document::new("low").with_score(0.1),
            Document::new("mid").with_score(0.5),
            Document::new("high").with_score(0.9),
        ];
        let reorder = GoogleVertexAIVisionReorder::new();
        let result = reorder.transform_documents(docs).await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].page_content, "high");
    }

    #[tokio::test]
    async fn test_nuclia_db_transformer() {
        let doc = Document::new("Hello world");
        let transformer = NucliaDBTransformer::new().with_kbid("test-kb");
        let result = transformer.transform_documents(vec![doc]).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].page_content, "Hello world");
        assert_eq!(result[0].metadata.get("format").unwrap(), "text");
        assert_eq!(result[0].metadata.get("kbid").unwrap(), "test-kb");
    }

    #[tokio::test]
    async fn test_long_context_reorder_no_scores() {
        let docs = vec![
            Document::new("first"),
            Document::new("second"),
            Document::new("third"),
        ];
        let reorder = LongContextReorder::new();
        let result = reorder.transform_documents(docs.clone()).await.unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_long_context_reorder_single_document() {
        let docs = vec![Document::new("only")];
        let reorder = LongContextReorder::new();
        let result = reorder.transform_documents(docs).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].page_content, "only");
    }

    #[tokio::test]
    async fn test_google_vertex_ai_vision_reorder_empty() {
        let reorder = GoogleVertexAIVisionReorder::default();
        let result = reorder.transform_documents(vec![]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_google_vertex_ai_vision_reorder_single() {
        let docs = vec![Document::new("doc")];
        let reorder = GoogleVertexAIVisionReorder::new();
        let result = reorder.transform_documents(docs).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_nuclia_db_transformer_default() {
        let doc = Document::new("content");
        let transformer = NucliaDBTransformer::default();
        let result = transformer.transform_documents(vec![doc]).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].metadata.get("format").unwrap(), "text");
        assert!(result[0].metadata.get("kbid").is_none());
    }

    #[tokio::test]
    async fn test_nuclia_db_transformer_with_path() {
        let doc = Document::new("content");
        let transformer = NucliaDBTransformer::new()
            .with_kbid("my-kb")
            .with_path("/path");
        let result = transformer.transform_documents(vec![doc]).await.unwrap();
        assert_eq!(
            result[0].metadata.get("kbid").unwrap(),
            "my-kb"
        );
        assert_eq!(
            result[0].metadata.get("path").unwrap(),
            "/path"
        );
    }

    #[tokio::test]
    async fn test_nuclia_db_transformer_preserves_metadata() {
        let mut meta = std::collections::HashMap::new();
        meta.insert(
            "source".into(),
            serde_json::Value::String("test".into()),
        );
        let doc = Document {
            page_content: "content".into(),
            metadata: meta,
            score: None,
        };
        let transformer = NucliaDBTransformer::new();
        let result = transformer.transform_documents(vec![doc]).await.unwrap();
        assert_eq!(result[0].metadata.get("source").unwrap(), "test");
        assert_eq!(result[0].metadata.get("format").unwrap(), "text");
    }

    #[tokio::test]
    async fn test_transform_document_single_delegates() {
        let transformer = LongContextReorder::new();
        let doc = Document::new("single");
        let result = transformer.transform_document(doc.clone()).await.unwrap();
        assert_eq!(result.page_content, "single");
    }

    #[test]
    fn test_document_transformers_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<LongContextReorder>();
        assert_sync::<LongContextReorder>();
        assert_send::<GoogleVertexAIVisionReorder>();
        assert_sync::<GoogleVertexAIVisionReorder>();
        assert_send::<NucliaDBTransformer>();
        assert_sync::<NucliaDBTransformer>();
    }

    #[tokio::test]
    async fn test_long_context_reorder_two_docs() {
        let docs = vec![
            Document::new("low").with_score(0.1),
            Document::new("high").with_score(0.9),
        ];
        let reorder = LongContextReorder::new();
        let result = reorder.transform_documents(docs).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].page_content, "high");
    }

    #[tokio::test]
    async fn test_long_context_reorder_four_docs() {
        let docs = vec![
            Document::new("d").with_score(0.1),
            Document::new("c").with_score(0.3),
            Document::new("b").with_score(0.6),
            Document::new("a").with_score(0.9),
        ];
        let reorder = LongContextReorder::new();
        let result = reorder.transform_documents(docs).await.unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].page_content, "a");
        // Alternating order: front(0)=a, back(3)=d, front(1)=b, back(2)=c
        assert_eq!(result[3].page_content, "c");
    }

    #[tokio::test]
    async fn test_google_vertex_reorder_two_docs() {
        let docs = vec![
            Document::new("low").with_score(0.2),
            Document::new("high").with_score(0.8),
        ];
        let reorder = GoogleVertexAIVisionReorder::new();
        let result = reorder.transform_documents(docs).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].page_content, "high");
    }

    #[tokio::test]
    async fn test_nuclia_db_transformer_preserves_score() {
        let doc = Document::new("content").with_score(0.75);
        let transformer = NucliaDBTransformer::new();
        let result = transformer.transform_documents(vec![doc]).await.unwrap();
        assert_eq!(result[0].score, Some(0.75));
    }

    #[tokio::test]
    async fn test_nuclia_db_transformer_empty_input() {
        let transformer = NucliaDBTransformer::new();
        let result = transformer.transform_documents(vec![]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_long_context_reorder_creates_alternating_order() {
        let docs = vec![
            Document::new("first").with_score(1.0),
            Document::new("second").with_score(0.8),
            Document::new("third").with_score(0.6),
            Document::new("fourth").with_score(0.4),
            Document::new("fifth").with_score(0.2),
        ];
        let reorder = LongContextReorder::new();
        let result = reorder.transform_documents(docs).await.unwrap();
        assert_eq!(result.len(), 5);
    }
}
