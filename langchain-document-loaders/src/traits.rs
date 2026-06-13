//! Core [`BaseLoader`] trait for loading documents from various sources.

use async_trait::async_trait;
use futures::stream::BoxStream;
use langchain_core::documents::Document;
use langchain_core::errors::Result;

/// Trait for document loaders that read content from a source and produce
/// [`Document`]s.
#[async_trait]
pub trait BaseLoader: Send + Sync {
    /// Loads all documents from the source.
    async fn load(&self) -> Result<Vec<Document>>;
    /// Loads documents lazily as a stream.
    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>>;
}

/// Extended loader trait for blob/binary sources with type and extension
/// metadata.
#[async_trait]
pub trait BaseBlobLoader: BaseLoader {
    /// Returns the MIME type or blob category.
    fn blob_type(&self) -> &str;
    /// Returns the list of allowed file extensions.
    fn allowed_extensions(&self) -> Vec<String>;
}
