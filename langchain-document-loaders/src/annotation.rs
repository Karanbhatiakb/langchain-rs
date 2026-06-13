/// Loader for annotation data sources.
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use langchain_core::documents::Document;
use langchain_core::errors::Result;
use crate::traits::BaseLoader;

#[derive(Debug, Clone)]
pub struct AnnotationLoader;

impl AnnotationLoader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnnotationLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseLoader for AnnotationLoader {
    async fn load(&self) -> Result<Vec<Document>> {
        Ok(Vec::new())
    }

    async fn lazy_load(&self) -> BoxStream<'static, Result<Document>> {
        futures::stream::empty().boxed()
    }
}
